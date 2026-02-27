use anyhow::{Result, anyhow};

use axum::{
    extract::{
        State,
        connect_info::ConnectInfo,
        ws::{WebSocket, WebSocketUpgrade},
    },
    response::IntoResponse,
};
use axum_extra::TypedHeader;
use tokio::time::timeout;

use std::ops::ControlFlow;
use std::{net::SocketAddr, time::Duration};

use shared::{
    data::user::User,
    http::server::GetUserDetailsResponse,
    messages::{
        chat::{ClientChatMessage, ClientConnectMessage},
        server::{ErrorMessage, ErrorType},
    },
};

use crate::{
    app_state::{AppSettings, AppState},
    chat::chat_session::ChatSession,
    websocket::WebSocketConnection,
};

const CLIENT_INACTIVITY_TIMEOUT: Duration = Duration::from_secs(60);

pub async fn handler(
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(app_state): State<AppState>,
) -> impl IntoResponse {
    let user_agent = if let Some(TypedHeader(user_agent)) = user_agent {
        user_agent.to_string()
    } else {
        String::from("Unknown browser")
    };
    tracing::info!("`{user_agent}` at {addr} connected.");

    ws.on_upgrade(move |socket| handle_socket(socket, addr, app_state))
}

async fn handle_socket(socket: WebSocket, addr: SocketAddr, app_state: AppState) {
    let mut conn = WebSocketConnection::establish(socket, addr, CLIENT_INACTIVITY_TIMEOUT);

    tracing::debug!("waiting for client to connect...");
    let mut session = match timeout(
        Duration::from_secs(30),
        wait_for_connect(&app_state, &mut conn),
    )
    .await
    {
        Err(e) => {
            tracing::error!("connection timeout: {}", e);
            return;
        }
        Ok(Err(e)) => {
            tracing::error!("unable to connect: {}", e);
            conn.send(
                &ErrorMessage {
                    error_type: ErrorType::Server,
                    message: e.to_string(),
                    must_disconnect: true,
                }
                .into(),
            )
            .await
            .unwrap_or_else(|e| tracing::error!("failed to send error message: {}", e));
            return;
        }
        Ok(Ok(p)) => p,
    };
    tracing::debug!("client connected");

    match session.run().await {
        Ok(()) => {
            if let Err(e) = handle_disconnect(session).await {
                tracing::error!("error handling disconnect for '{addr}': {e}")
            }
        }
        Err(e) => tracing::error!("error running game: {e}"),
    }

    // db::game_sessions::end_session(&app_state.db_pool, &character_id)
    //     .await
    //     .unwrap_or_else(|e| tracing::error!("error ending session for '{character_id}': {e}"));

    // returning from the handler closes the websocket connection
    tracing::info!("websocket context '{addr}' destroyed");
}

async fn wait_for_connect<'a>(
    app_state: &AppState,
    conn: &'a mut WebSocketConnection,
) -> Result<ChatSession<'a>> {
    loop {
        match conn.poll_receive() {
            ControlFlow::Continue(Some(ClientChatMessage::Connect(msg))) => {
                return handle_connect(app_state, conn, *msg).await;
            }
            ControlFlow::Break(_) => {
                return Err(anyhow::format_err!("disconnected"));
            }
            _ => {}
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

async fn handle_connect<'a>(
    app_state: &AppState,
    conn: &'a mut WebSocketConnection,
    msg: ClientConnectMessage,
) -> Result<ChatSession<'a>> {
    tracing::info!("connect: {}", msg.user_id);
    let user = authorize_jwt(&app_state.app_settings, &msg.jwt).await?;
    Ok(ChatSession::new(conn, user))
}

async fn handle_disconnect<'a>(session: ChatSession<'a>) -> Result<()> {
    Ok(())
}

async fn authorize_jwt(app_settings: &AppSettings, token: &str) -> anyhow::Result<User> {
    let res = reqwest::Client::new()
        .get(format!("{}/account/me", app_settings.backend_url))
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .send()
        .await?;

    if !res.status().is_success() {
        let err = res.text().await?;
        anyhow::bail!("Server API error: {}", err);
    }

    Ok(res
        .json::<GetUserDetailsResponse>()
        .await?
        .user_details
        .user)
}
