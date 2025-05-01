use anyhow::Result;

use axum::{
    extract::{
        connect_info::ConnectInfo,
        ws::{WebSocket, WebSocketUpgrade},
    },
    response::IntoResponse,
};
use axum_extra::TypedHeader;
use tokio::{task::yield_now, time::timeout};

use std::{net::SocketAddr, time::Duration};
use std::{ops::ControlFlow, vec};

use shared::{
    data::{
        character::CharacterSize,
        item::{
            AffixEffect, AffixEffectType, AffixType, ItemAffix, ItemCategory, ItemRarity,
            ItemSpecs, ItemStat, WeaponSpecs,
        },
        player::{CharacterSpecs, PlayerInventory, PlayerSpecs},
        skill::{Range, Shape, SkillSpecs, TargetType},
    },
    messages::{
        client::{ClientConnectMessage, ClientMessage},
        server::ConnectMessage,
    },
};

use crate::game::{systems::weapon::make_weapon_skill, world::WorldBlueprint, GameInstance};
use crate::websocket::WebSocketConnection;

const CLIENT_INACTIVITY_TIMEOUT: Duration = Duration::from_secs(60);

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    let user_agent = if let Some(TypedHeader(user_agent)) = user_agent {
        user_agent.to_string()
    } else {
        String::from("Unknown browser")
    };
    tracing::info!("`{user_agent}` at {addr} connected.");

    ws.on_upgrade(move |socket| handle_socket(socket, addr))
}

async fn handle_socket(socket: WebSocket, who: SocketAddr) {
    let mut conn = WebSocketConnection::establish(socket, who, CLIENT_INACTIVITY_TIMEOUT);

    tracing::debug!("waiting for client to connect...");
    let player = match timeout(Duration::from_secs(30), wait_for_connect(&mut conn)).await {
        Err(e) => {
            tracing::error!("connection timeout: {}", e);
            return;
        }
        Ok(Err(e)) => {
            tracing::error!("unable to connect: {}", e);
            return;
        }
        Ok(Ok(p)) => p,
    };
    tracing::debug!("client connected");

    tracing::debug!("starting the game...");
    match WorldBlueprint::load_from_file("worlds/forest.json".into()).await {
        Ok(world_blueprint) => {
            let mut game = GameInstance::new(&mut conn, player, world_blueprint);
            if let Err(e) = game.run().await {
                tracing::error!("error running game: {e}");
            }
        }
        Err(e) => tracing::error!("failed to load world: {e}"),
    }

    // returning from the handler closes the websocket connection
    tracing::info!("websocket context {who} destroyed");
}

async fn wait_for_connect(conn: &mut WebSocketConnection) -> Result<PlayerSpecs> {
    loop {
        match conn.poll_receive() {
            ControlFlow::Continue(Some(ClientMessage::Connect(m))) => {
                return handle_connect(conn, m).await;
            }
            ControlFlow::Break(_) => {
                return Err(anyhow::format_err!("disconnected"));
            }
            _ => {}
        }
        yield_now().await;
    }
}

async fn handle_connect(
    conn: &mut WebSocketConnection,
    msg: ClientConnectMessage,
) -> Result<PlayerSpecs> {
    // TODO: verify if user exist, is already playing, get basic data etc
    tracing::info!("Connect: {:?}", msg);
    conn.send(
        &ConnectMessage {
            greeting: msg.bearer.clone(),
            value: 42,
        }
        .into(),
    )
    .await?;

    let weapon = ItemSpecs {
        name: "Shortsword".to_string(),
        description: "Fasty Slicy".to_string(),
        icon: "items/shortsword.webp".to_string(),
        item_level: 1,
        rarity: ItemRarity::Normal,
        affixes: Vec::new(),
        item_category: ItemCategory::Weapon(WeaponSpecs {
            base_cooldown: 1.0,
            cooldown: 1.0,
            range: Range::Melee,
            shape: Shape::Single,
            base_min_damage: 3.0,
            min_damage: 3.0,
            base_max_damage: 7.0,
            max_damage: 7.0,
        }),
    };

    Ok(PlayerSpecs {
        character_specs: CharacterSpecs {
            name: msg.bearer.clone(),
            portrait: String::from("adventurers/human_male_2.webp"),
            size: CharacterSize::Small,
            position_x: 0,
            position_y: 0,
            max_health: 100.0,
            health_regen: 1.0,
        },
        skill_specs: vec![
            make_weapon_skill(&weapon).unwrap_or_default(),
            SkillSpecs {
                name: String::from("Fireball"),
                description: "A throw of mighty fireball, burning multiple enemies".to_string(),
                icon: String::from("skills/fireball2.svg"),
                cooldown: 5.0,
                mana_cost: 20.0,
                min_damages: 10.0,
                max_damages: 30.0,
                range: Range::Distance,
                target_type: TargetType::Enemy,
                shape: Shape::Square4,
                upgrade_level: 1,
                next_upgrade_cost: 10.0,
            },
            SkillSpecs {
                name: String::from("Heal"),
                description: "A minor healing spell for yourself".to_string(),
                icon: String::from("skills/heal.svg"),
                cooldown: 30.0,
                mana_cost: 20.0,
                min_damages: -20.0,
                max_damages: -20.0,
                range: Range::Melee,
                target_type: TargetType::Me,
                shape: Shape::Single,
                upgrade_level: 1,
                next_upgrade_cost: 10.0,
            },
        ],
        level: 1,
        experience_needed: 10.0,
        max_mana: 100.0,
        mana_regen: 3.0,
        auto_skills: vec![true, false, false],
        inventory: PlayerInventory {
            weapon_specs: Some(weapon),
            max_bag_size: 40,
            bag: vec![
                ItemSpecs {
                    name: "Battleaxe".to_string(),
                    description: "A shiny thing".to_string(),
                    icon: "items/battleaxe.webp".to_string(),
                    item_level: 2,
                    rarity: ItemRarity::Magic,
                    item_category: ItemCategory::Weapon(WeaponSpecs {
                        base_cooldown: 1.2,
                        cooldown: 1.2,
                        range: Range::Melee,
                        shape: Shape::Single,
                        base_min_damage: 4.0,
                        min_damage: 4.0,
                        base_max_damage: 8.0,
                        max_damage: 8.0,
                    }),
                    affixes: vec![ItemAffix {
                        name: "Painful".to_string(),
                        family: "inc_damage".to_string(),
                        affix_type: AffixType::Prefix,
                        affix_level: 1,
                        effects: vec![AffixEffect {
                            stat: ItemStat::AttackDamage,
                            effect_type: AffixEffectType::Multiplier,
                            value: 0.1,
                        }],
                    }],
                },
                ItemSpecs {
                    name: "Shortsword".to_string(),
                    description: "Fasty Slicy".to_string(),
                    icon: "items/shortsword.webp".to_string(),
                    item_level: 1,
                    rarity: ItemRarity::Rare,
                    item_category: ItemCategory::Weapon(WeaponSpecs {
                        base_cooldown: 1.0,
                        cooldown: 1.0,
                        range: Range::Melee,
                        shape: Shape::Single,
                        base_min_damage: 3.0,
                        min_damage: 3.0,
                        base_max_damage: 7.0,
                        max_damage: 7.0,
                    }),
                    affixes: vec![
                        ItemAffix {
                            name: "Painful".to_string(),
                            family: "inc_damage".to_string(),
                            affix_type: AffixType::Prefix,
                            affix_level: 1,
                            effects: vec![AffixEffect {
                                stat: ItemStat::AttackDamage,
                                effect_type: AffixEffectType::Multiplier,
                                value: 0.1,
                            }],
                        },
                        ItemAffix {
                            name: "Merciless".to_string(),
                            family: "inc_damage".to_string(),
                            affix_type: AffixType::Prefix,
                            affix_level: 1,
                            effects: vec![
                                AffixEffect {
                                    stat: ItemStat::MinAttackDamage,
                                    effect_type: AffixEffectType::Flat,
                                    value: 1.0,
                                },
                                AffixEffect {
                                    stat: ItemStat::MaxAttackDamage,
                                    effect_type: AffixEffectType::Flat,
                                    value: 2.0,
                                },
                            ],
                        },
                        ItemAffix {
                            name: "Greedy".to_string(),
                            family: "gold".to_string(),
                            affix_type: AffixType::Suffix,
                            affix_level: 1,
                            effects: vec![AffixEffect {
                                stat: ItemStat::GoldFind,
                                effect_type: AffixEffectType::Multiplier,
                                value: 0.3,
                            }],
                        },
                        ItemAffix {
                            name: "Fast".to_string(),
                            family: "inc_speed".to_string(),
                            affix_type: AffixType::Prefix,
                            affix_level: 1,
                            effects: vec![AffixEffect {
                                stat: ItemStat::AttackSpeed,
                                effect_type: AffixEffectType::Multiplier,
                                value: 0.1,
                            }],
                        },
                    ],
                },
                ItemSpecs {
                    name: "Stabby the First".to_string(),
                    description: "Most Unique Fasty Slicy".to_string(),
                    icon: "items/shortsword.webp".to_string(),
                    item_level: 1,
                    rarity: ItemRarity::Unique,
                    item_category: ItemCategory::Weapon(WeaponSpecs {
                        base_cooldown: 1.0,
                        cooldown: 0.8,
                        range: Range::Melee,
                        shape: Shape::Single,
                        base_min_damage: 1.0,
                        min_damage: 1.0,
                        base_max_damage: 13.0,
                        max_damage: 13.0,
                    }),
                    affixes: Vec::new(),
                },
            ],
        },
    })
}
