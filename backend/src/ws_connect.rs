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
        item::{ItemCategory, ItemSpecs, WeaponSpecs},
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
        item_category: ItemCategory::Weapon(WeaponSpecs {
            base_cooldown: 1.0,
            cooldown: 1.0,
            range: Range::Front,
            shape: Shape::Single,
            base_min_damages: 3.0,
            min_damages: 3.0,
            base_max_damages: 7.0,
            max_damages: 7.0,
            magic_prefixes: Vec::new(),
            magic_suffixes: Vec::new(),
        }),
    };

    Ok(PlayerSpecs {
        character_specs: CharacterSpecs {
            name: msg.bearer.clone(),
            portrait: String::from("adventurers/human_male_2.webp"),
            max_health: 100.0,
            health_regen: 1.0,
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
                    range: Range::Middle,
                    target_type: TargetType::Enemy,
                    shape: Shape::Square4,
                },
                SkillSpecs {
                    name: String::from("Heal"),
                    description: "A minor healing spell for yourself".to_string(),
                    icon: String::from("skills/heal.svg"),
                    cooldown: 30.0,
                    mana_cost: 20.0,
                    min_damages: -20.0,
                    max_damages: -20.0,
                    range: Range::Front,
                    target_type: TargetType::Me,
                    shape: Shape::Single,
                },
            ],
        },
        level: 1,
        experience_needed: 10.0,
        max_mana: 100.0,
        mana_regen: 3.0,
        auto_skills: vec![true, false, false],
        inventory: PlayerInventory {
            weapon_specs: Some(weapon),
            bag: vec![ItemSpecs {
                name: "Battleaxe".to_string(),
                description: "A shiny thing".to_string(),
                icon: "items/battleaxe.webp".to_string(),
                item_level: 2,
                item_category: ItemCategory::Weapon(WeaponSpecs {
                    base_cooldown: 1.2,
                    cooldown: 1.2,
                    range: Range::Front,
                    shape: Shape::Single,
                    base_min_damages: 4.0,
                    min_damages: 4.0,
                    base_max_damages: 8.0,
                    max_damages: 8.0,
                    magic_prefixes: Vec::new(),
                    magic_suffixes: Vec::new(),
                }),
            }],
        },
    })
}
