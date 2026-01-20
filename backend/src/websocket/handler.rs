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
use tokio::time::timeout;

use std::ops::ControlFlow;
use std::{
    net::SocketAddr,
    time::{Duration, Instant},
};

use shared::messages::{
    client::{ClientConnectMessage, ClientMessage},
    server::{ErrorMessage, ErrorType},
};

use crate::{
    app_state::AppState,
    auth,
    db::{self},
    game::{
        sessions::{Session, SessionsStore},
        systems::sessions_controller,
        GameInstance,
    },
    rest::AppError,
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

    let game = GameInstance::new(
        &mut conn,
        &session.character_id,
        &mut session.game_data,
        app_state.db_pool.clone(),
        app_state.master_store,
        app_state.sessions_store.clone(),
    );

    let character_id = session.character_id;

    match game.run().await {
        Ok(()) => {
            if let Err(e) = handle_disconnect(&app_state.sessions_store, session).await {
                tracing::error!("error handling disconnect for '{addr}': {e}")
            }
        }
        Err(e) => tracing::error!("error running game: {e}"),
    }

    db::game_sessions::end_session(&app_state.db_pool, &character_id)
        .await
        .unwrap_or_else(|e| tracing::error!("error ending session for '{character_id}': {e}"));

    // returning from the handler closes the websocket connection
    tracing::info!("websocket context '{addr}' destroyed");
}

async fn wait_for_connect(app_state: &AppState, conn: &mut WebSocketConnection) -> Result<Session> {
    loop {
        match conn.poll_receive() {
            ControlFlow::Continue(Some(ClientMessage::Connect(msg))) => {
                return handle_connect(app_state, msg).await;
            }
            ControlFlow::Break(_) => {
                return Err(anyhow::format_err!("disconnected"));
            }
            _ => {}
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

async fn handle_connect(
    app_state: &AppState,
    // conn: &mut WebSocketConnection,
    msg: ClientConnectMessage,
) -> Result<Session> {
    tracing::info!("connect: {:?}", msg);

    let user_id = auth::authorize_jwt(&app_state.app_settings, &msg.jwt)
        .ok_or(AppError::Unauthorized("invalid token".to_string()))?;

    let user_character = db::characters::read_character(&app_state.db_pool, &msg.character_id)
        .await?
        .ok_or(AppError::NotFound)?;

    if user_character.user_id != user_id {
        return Err(AppError::NotFound.into());
    }

    // conn.send(&ConnectMessage {}.into()).await?;

    // Must be the last thing we do because we cannot fail after
    let session = sessions_controller::create_session(
        &app_state.db_pool,
        &app_state.sessions_store,
        &app_state.master_store,
        user_character,
        msg.area_config,
    )
    .await?;

    Ok(session)
}

async fn handle_disconnect(sessions_store: &SessionsStore, mut session: Session) -> Result<()> {
    let end_quest = session.game_data.area_state.read().end_quest;

    session.last_active = Instant::now();

    if !end_quest {
        sessions_store
            .sessions
            .lock()
            .unwrap()
            .insert(session.character_id, session);
    }

    Ok(())
}
