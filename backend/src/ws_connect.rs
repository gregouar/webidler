use anyhow::Result;

use axum::{
    extract::{
        connect_info::ConnectInfo,
        ws::{WebSocket, WebSocketUpgrade},
    },
    response::IntoResponse,
};
use axum_extra::TypedHeader;

use std::net::SocketAddr;
use std::ops::ControlFlow;

use shared::client_messages::{ClientConnectMessage, ClientMessage};

use crate::game;
use crate::websocket::WebSocketConnection;

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
    log::info!("`{user_agent}` at {addr} connected.");

    ws.on_upgrade(move |socket| handle_socket(socket, addr))
}

async fn handle_socket(socket: WebSocket, who: SocketAddr) {
    let mut conn = WebSocketConnection::establish(socket, who);

    if let Err(e) = wait_for_connect(&mut conn).await {
        log::info!("Invalid connect {}", e);
        return;
    }

    if let Err(e) = game::run(&mut conn).await {
        log::error!("Error running game: {e}");
    }

    // returning from the handler closes the websocket connection
    log::info!("Websocket context {who} destroyed");
}

async fn wait_for_connect(conn: &mut WebSocketConnection) -> Result<()> {
    loop {
        match conn.poll_receive() {
            ControlFlow::Continue(Some(ClientMessage::Connect(m))) => {
                return handle_connect(m);
            }
            ControlFlow::Break(_) => {
                return Err(anyhow::format_err!("disconnected"));
            }
            _ => {}
        }
    }
}

fn handle_connect(msg: ClientConnectMessage) -> Result<()> {
    // TODO: verify if user exist, is already playing, get basic data etc
    log::info!("Connect: {:?}", msg);
    Ok(())
}
