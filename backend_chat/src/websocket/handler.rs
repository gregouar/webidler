use anyhow::Result;

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
    websocket::{self, WebSocketReceiver},
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
    let (mut ws_sender, mut ws_receiver) =
        websocket::establish(socket, addr, CLIENT_INACTIVITY_TIMEOUT);

    tracing::debug!("waiting for client to connect...");
    let session = match timeout(
        Duration::from_secs(30),
        wait_for_connect(&app_state, &mut ws_receiver),
    )
    .await
    {
        Err(e) => {
            tracing::error!("connection timeout: {}", e);
            return;
        }
        Ok(Err(e)) => {
            tracing::error!("unable to connect: {}", e);
            ws_sender
                .send(
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

    match session.run(ws_sender, ws_receiver).await {
        Ok(()) => {
            // if let Err(e) = handle_disconnect(session).await {
            //     tracing::error!("error handling disconnect for '{addr}': {e}")
            // }
        }
        Err(e) => tracing::error!("error running game: {e}"),
    }

    // app_state.chat_state.reply_map.remove(&session_id);

    // db::game_sessions::end_session(&app_state.db_pool, &character_id)
    //     .await
    //     .unwrap_or_else(|e| tracing::error!("error ending session for '{character_id}': {e}"));

    // returning from the handler closes the websocket connection
    tracing::info!("websocket context '{addr}' destroyed");
}

async fn wait_for_connect(
    app_state: &AppState,
    ws_receiver: &mut WebSocketReceiver,
) -> Result<ChatSession> {
    loop {
        match ws_receiver.block_receive().await {
            ControlFlow::Continue(ClientChatMessage::Connect(msg)) => {
                return handle_connect(app_state, *msg).await;
            }
            ControlFlow::Break(_) => {
                return Err(anyhow::format_err!("disconnected"));
            }
            _ => {}
        }
    }
}

async fn handle_connect<'a>(
    app_state: &AppState,
    msg: ClientConnectMessage,
) -> Result<ChatSession> {
    let user = authorize_jwt(&app_state.app_settings, &msg.jwt).await?;
    tracing::info!("connect: {}", user.user_id);
    Ok(ChatSession::new(app_state.chat_state.clone(), user))
}

// async fn handle_disconnect<'a>(session: ChatSession<'a>) -> Result<()> {
//     Ok(())
// }

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
