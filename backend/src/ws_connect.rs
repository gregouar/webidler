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

use std::ops::ControlFlow;
use std::{net::SocketAddr, time::Duration};

use shared::messages::{
    client::{ClientConnectMessage, ClientMessage},
    server::ConnectMessage,
};

use crate::game::GameInstance;
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
    match timeout(Duration::from_secs(30), wait_for_connect(&mut conn)).await {
        Err(e) => {
            tracing::error!("connection timeout: {}", e);
            return;
        }
        Ok(Err(e)) => {
            tracing::error!("unable to connect: {}", e);
            return;
        }
        Ok(Ok(_)) => {
            tracing::debug!("client connected");
        }
    }

    tracing::debug!("starting the game...");
    let mut game = GameInstance::new(&mut conn);
    if let Err(e) = game.run().await {
        tracing::error!("error running game: {e}");
    }

    // returning from the handler closes the websocket connection
    tracing::info!("websocket context {who} destroyed");
}

async fn wait_for_connect(conn: &mut WebSocketConnection) -> Result<()> {
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

async fn handle_connect(conn: &mut WebSocketConnection, msg: ClientConnectMessage) -> Result<()> {
    // TODO: verify if user exist, is already playing, get basic data etc
    tracing::info!("Connect: {:?}", msg);
    conn.send(
        &ConnectMessage {
            greeting: String::from("Poupou"),
            value: 42,
        }
        .into(),
    )
    .await?;
    Ok(())
}
