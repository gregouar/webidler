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

pub struct GameInstance<'a> {
    client_conn: &'a mut WebSocketConnection,
    loop_counter: i32,
    // todo: map infos, player, monsters, etc
}

impl<'a> GameInstance<'a> {
    pub fn new(client_conn: &'a mut WebSocketConnection) -> Self {
        GameInstance::<'a> {
            client_conn,
            loop_counter: 0,
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        let mut last_time = Instant::now();
        loop {
            self.loop_counter += 1;

            if let ControlFlow::Break(_) = self.handle_client_events().await {
                break;
            }

            self.sync_client().await?;

            // Wait for next tick
            let duration = last_time.elapsed();
            if duration < LOOP_MIN_PERIOD {
                tokio::time::sleep(LOOP_MIN_PERIOD - duration).await;
            }
            last_time = Instant::now();
        }

        Ok(())
    }

    /// Handle client events, return whether the game should stop or continue
    async fn handle_client_events(&mut self) -> ControlFlow<(), ()> {
        // TODO: Should We limit the amount of events we handle in one loop?
        // for _ in 1..10 {
        loop {
            match self.client_conn.poll_receive() {
                ControlFlow::Continue(Some(m)) => self.handle_client_message(m),
                ControlFlow::Continue(None) => return ControlFlow::Continue(()), // No more messages
                ControlFlow::Break(_) => return ControlFlow::Break(()), // Connection closed
            }
            yield_now().await;
        }
    }

    fn handle_client_message(&mut self, msg: ClientMessage) {
        match msg {
            ClientMessage::Heartbeat => {
                tracing::debug!("heartbeat");
            }
            ClientMessage::Test(m) => {
                tracing::info!("test: {:?}", m)
            }
            _ => {
                tracing::warn!("received unexpected message: {:?}", msg)
            }
        }
    }

    /// Send whole world state to client
    async fn sync_client(&mut self) -> Result<()> {
        self.client_conn
            .send(&ServerMessage::Update(UpdateMessage {
                value: self.loop_counter,
            }))
            .await
    }
}
