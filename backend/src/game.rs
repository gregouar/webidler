use anyhow::Result;

use tokio::task::yield_now;

use std::{
    ops::ControlFlow,
    time::{Duration, Instant},
};

use shared::{
    client_messages::ClientMessage,
    server_messages::{ServerMessage, UpdateMessage},
};

use crate::websocket::WebSocketConnection;

const LOOP_MIN_PERIOD: Duration = Duration::from_millis(100);
pub struct GameInfo {}

pub async fn run(conn: &mut WebSocketConnection) -> Result<()> {
    let mut last_time = Instant::now();
    let mut i = 0;
    loop {
        i += 1;

        if !handle_client_events(conn).await {
            break;
        }

        sync_client(conn, i).await?;

        // Wait for next tick
        let duration = last_time.elapsed();
        if duration < LOOP_MIN_PERIOD {
            tokio::time::sleep(LOOP_MIN_PERIOD - duration).await;
        }
        last_time = Instant::now();
    }

    Ok(())
}

/// Handle client events, return whether the game should stop
async fn handle_client_events(conn: &mut WebSocketConnection) -> bool {
    // TODO: Should We limit the amount of events we handle in one loop?
    // for _ in 1..10 {
    loop {
        match conn.poll_receive() {
            ControlFlow::Continue(Some(m)) => handle_client_message(m),
            ControlFlow::Continue(None) => return true,
            ControlFlow::Break(_) => return false,
        }
        yield_now().await;
    }
}

fn handle_client_message(msg: ClientMessage) {
    match msg {
        ClientMessage::Heartbeat => {
            tracing::debug!("heartbeat");
        }
        ClientMessage::Test(m) => {
            tracing::info!("Test: {:?}", m)
        }
        _ => {
            tracing::warn!("Received unexpected message: {:?}", msg)
        }
    }
}

async fn sync_client(conn: &mut WebSocketConnection, i: i32) -> Result<()> {
    conn.send(&ServerMessage::Update(UpdateMessage { value: i }))
        .await
}
