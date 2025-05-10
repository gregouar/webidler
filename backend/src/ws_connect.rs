use anyhow::Result;

use axum::{
    extract::{
        connect_info::ConnectInfo,
        ws::{WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use axum_extra::TypedHeader;
use tokio::{task::yield_now, time::timeout};

use std::{net::SocketAddr, path::Path, time::Duration};
use std::{ops::ControlFlow, vec};

use shared::{
    data::{
        character::CharacterSize,
        item::{ItemRarity, ItemSpecs},
        player::{CharacterSpecs, PlayerInventory, PlayerSpecs, PlayerState},
        skill::{
            DamageType, Range, Shape, SkillEffect, SkillEffectType, SkillSpecs, SkillType,
            TargetType,
        },
    },
    messages::{
        client::{ClientConnectMessage, ClientMessage},
        server::ConnectMessage,
    },
};

use crate::game::{
    data::{master_store::MasterStore, DataInit},
    systems::{items_controller, player_controller},
    GameInstance,
};
use crate::websocket::WebSocketConnection;

const CLIENT_INACTIVITY_TIMEOUT: Duration = Duration::from_secs(60);

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(master_store): State<MasterStore>,
) -> impl IntoResponse {
    let user_agent = if let Some(TypedHeader(user_agent)) = user_agent {
        user_agent.to_string()
    } else {
        String::from("Unknown browser")
    };
    tracing::info!("`{user_agent}` at {addr} connected.");

    ws.on_upgrade(move |socket| handle_socket(socket, addr, master_store))
}

async fn handle_socket(socket: WebSocket, who: SocketAddr, master_store: MasterStore) {
    let mut conn = WebSocketConnection::establish(socket, who, CLIENT_INACTIVITY_TIMEOUT);

    tracing::debug!("waiting for client to connect...");
    let mut player_specs = match timeout(Duration::from_secs(30), wait_for_connect(&mut conn)).await
    {
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

    tracing::debug!("loading the game...");
    let world_blueprint = match master_store
        .world_blueprints_store
        .get(Path::new("forest.json"))
        .cloned() // TODO: Avoid clone?
    {
        Some(world_blueprint) => world_blueprint,
        None => {
            tracing::error!("couldn't load world: forest.json");
            return;
        }
    };

    let mut player_state = PlayerState::init(&player_specs); // How to avoid this?

    if let Some(base_weapon) = master_store.items_table.entries.get("shortsword").cloned() {
        player_controller::equip_item(
            &mut player_specs,
            &mut player_state,
            items_controller::update_item_specs(ItemSpecs {
                base: base_weapon,
                level: 1,
                rarity: ItemRarity::Normal,
                affixes: Vec::new(),
                weapon_specs: None,
                armor_specs: None,
            }),
        );
    }

    tracing::debug!("starting the game...");

    let mut game = GameInstance::new(&mut conn, player_specs, world_blueprint, master_store);
    if let Err(e) = game.run().await {
        tracing::error!("error running game: {e}");
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

    let player_specs = PlayerSpecs {
        character_specs: CharacterSpecs {
            name: msg.bearer.clone(),
            portrait: String::from("adventurers/human_male_2.webp"),
            size: CharacterSize::Small,
            position_x: 0,
            position_y: 0,
            max_health: 100.0,
            health_regen: 1.0,
        },
        skills_specs: vec![
            SkillSpecs {
                name: String::from("Fireball"),
                description: "A throw of mighty fireball, burning multiple enemies".to_string(),
                icon: String::from("skills/fireball2.svg"),
                skill_type: SkillType::Spell,
                cooldown: 5.0,
                mana_cost: 20.0,
                upgrade_level: 1,
                next_upgrade_cost: 10.0,
                effects: vec![SkillEffect {
                    range: Range::Distance,
                    target_type: TargetType::Enemy,
                    shape: Shape::Square4,
                    effect_type: SkillEffectType::FlatDamage {
                        min: 4.0,
                        max: 12.0,
                        damage_type: DamageType::Fire,
                    },
                }],
            },
            SkillSpecs {
                name: String::from("Heal"),
                description: "A minor healing spell for yourself".to_string(),
                icon: String::from("skills/heal.svg"),
                skill_type: SkillType::Spell,
                cooldown: 30.0,
                mana_cost: 20.0,
                upgrade_level: 1,
                next_upgrade_cost: 10.0,
                effects: vec![SkillEffect {
                    range: Range::Melee,
                    target_type: TargetType::Me,
                    shape: Shape::Single,
                    effect_type: SkillEffectType::Heal {
                        min: 20.0,
                        max: 20.0,
                    },
                }],
            },
        ],
        level: 1,
        experience_needed: 10.0,
        max_mana: 100.0,
        mana_regen: 3.0,
        auto_skills: vec![false, false],
        inventory: PlayerInventory {
            weapon_specs: None,
            helmet_specs: None,
            max_bag_size: 40,
            bag: vec![],
        },
    };

    Ok(player_specs)
}
