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
    server::ConnectMessage,
    SessionId,
};

use crate::{
    auth,
    db::{self, DbPool},
    game::{
        data::master_store::MasterStore,
        sessions::{Session, SessionsStore},
        systems::sessions_controller,
        GameInstance,
    },
    websocket::WebSocketConnection,
};

const CLIENT_INACTIVITY_TIMEOUT: Duration = Duration::from_secs(60);

pub async fn handler(
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(db_pool): State<DbPool>,
    State(sessions_store): State<SessionsStore>,
    State(master_store): State<MasterStore>,
) -> impl IntoResponse {
    let user_agent = if let Some(TypedHeader(user_agent)) = user_agent {
        user_agent.to_string()
    } else {
        String::from("Unknown browser")
    };
    tracing::info!("`{user_agent}` at {addr} connected.");

    ws.on_upgrade(move |socket| handle_socket(socket, addr, db_pool, sessions_store, master_store))
}

async fn handle_socket(
    socket: WebSocket,
    addr: SocketAddr,
    db_pool: DbPool,
    sessions_store: SessionsStore,
    master_store: MasterStore,
) {
    let mut conn = WebSocketConnection::establish(socket, addr, CLIENT_INACTIVITY_TIMEOUT);

    tracing::debug!("waiting for client to connect...");
    let (session_id, mut session) = match timeout(
        Duration::from_secs(30),
        wait_for_connect(&db_pool, &master_store, &sessions_store, &mut conn),
    )
    .await
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

    tracing::debug!("starting the game...");
    let game = GameInstance::new(
        &mut conn,
        &session.character_id,
        &session_id,
        &mut session.game_data,
        db_pool.clone(),
        master_store,
    );
    match game.run().await {
        Ok(()) => {
            if let Err(e) = handle_disconnect(&db_pool, &sessions_store, session_id, session).await
            {
                tracing::error!("error handling disconnect for '{addr}': {e}")
            }
        }
        Err(e) => tracing::error!("error running game: {e}"),
    }

    // returning from the handler closes the websocket connection
    tracing::info!("websocket context '{addr}' destroyed");
}

async fn wait_for_connect(
    db_pool: &DbPool,
    master_store: &MasterStore,
    sessions_store: &SessionsStore,
    conn: &mut WebSocketConnection,
) -> Result<(SessionId, Session)> {
    loop {
        match conn.poll_receive() {
            ControlFlow::Continue(Some(ClientMessage::Connect(msg))) => {
                return handle_connect(db_pool, sessions_store, master_store, conn, msg).await;
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
    db_pool: &DbPool,
    sessions_store: &SessionsStore,
    master_store: &MasterStore,
    conn: &mut WebSocketConnection,
    msg: ClientConnectMessage,
) -> Result<(SessionId, Session)> {
    tracing::info!("connect: {:?}", msg);

    let user_id = match auth::authorize_jwt(&msg.jwt) {
        Some(user_id) => user_id,
        None => return Err(anyhow::anyhow!("invalid jwt")),
    };

    let user_character = db::characters::read_character(db_pool, &msg.character_id)
        .await?
        .ok_or(anyhow::anyhow!("character not found"))?;

    if user_character.user_id != user_id {
        return Err(anyhow::anyhow!("character not found"));
    }

    // TODO: load character
    // TODO: verify character/session belongs to user !!!!

    let (session_id, session) = match (msg.session_id, msg.session_key) {
        (Some(session_id), Some(session_key)) => {
            sessions_controller::resume_session(sessions_store, session_id, session_key).await?
        }
        _ => {
            let area_id: String = msg.area_id.ok_or(anyhow::anyhow!("missing area id"))?;
            sessions_controller::create_session(db_pool, master_store, user_character, &area_id)
                .await?
        }
    };

    conn.send(
        &ConnectMessage {
            session_id,
            session_key: session.session_key,
        }
        .into(),
    )
    .await?;

    Ok((session_id, session))
}

async fn handle_disconnect(
    db_pool: &DbPool,
    sessions_store: &SessionsStore,
    session_id: SessionId,
    mut session: Session,
) -> Result<()> {
    let end_quest = session.game_data.area_state.read().end_quest;

    session.last_active = Instant::now();

    if end_quest {
        sessions_controller::end_session(db_pool, &session_id).await?;
    } else {
        sessions_store
            .sessions
            .lock()
            .unwrap()
            .insert(session_id, session);
    }

    Ok(())
}
