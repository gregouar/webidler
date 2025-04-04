use anyhow::Result;

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
    'main: loop {
        // Handle client events
        // TODO: Should We limit the amount of events we handle in one loop?
        // for _ in 1..10 {
        loop {
            match conn.poll_receive() {
                ControlFlow::Continue(Some(m)) => handle_client_message(m),
                ControlFlow::Continue(None) => break,
                ControlFlow::Break(_) => break 'main,
            }
        }

        // Sync client
        i += 1;
        conn.send(&ServerMessage::Update(UpdateMessage { value: i }))
            .await?;

        // Wait for next tick
        let duration = last_time.elapsed();
        if duration < LOOP_MIN_PERIOD {
            tokio::time::sleep(LOOP_MIN_PERIOD - duration).await;
        }
        last_time = Instant::now();
    }

    Ok(())
}

fn handle_client_message(msg: ClientMessage) {
    match msg {
        ClientMessage::Heartbeat => {}
        ClientMessage::Test(m) => {
            log::info!("Test: {:?}", m)
        }
        _ => {
            log::info!("Received unexpected message: {:?}", msg)
        }
    }
}
