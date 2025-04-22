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
    data::{CharacterSpecs, PlayerSpecs, SkillSpecs},
    messages::{
        client::{ClientConnectMessage, ClientMessage},
        server::ConnectMessage,
    },
};

use crate::game::{world::WorldBlueprint, GameInstance};
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

    match WorldBlueprint::load_from_file("./data/worlds/forest.json".into()).await {
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
    Ok(PlayerSpecs {
        character_specs: CharacterSpecs {
            // identifier: 0,
            name: msg.bearer.clone(),
            portrait: String::from("adventurers/human_male_2.webp"),
            max_health: 100.0,
            health_regen: 1.0,
            skill_specs: vec![
                SkillSpecs {
                    name: String::from("Weapon Attack"),
                    icon: String::from("icons/attack.svg"),
                    cooldown: 1.0,
                    mana_cost: 0.0,
                    min_damages: 3.0,
                    max_damages: 7.0,
                },
                SkillSpecs {
                    name: String::from("Fireball"),
                    icon: String::from("icons/fireball2.svg"),
                    cooldown: 5.0,
                    mana_cost: 20.0,
                    min_damages: 10.0,
                    max_damages: 30.0,
                },
                SkillSpecs {
                    name: String::from("Heal"),
                    icon: String::from("icons/heal.svg"),
                    cooldown: 30.0,
                    mana_cost: 20.0,
                    min_damages: -20.0,
                    max_damages: -20.0,
                },
            ],
        },
        max_mana: 100.0,
        mana_regen: 3.0,
        auto_skills: vec![true, false, false],
    })
}
