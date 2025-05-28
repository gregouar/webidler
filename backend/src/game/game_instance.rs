use anyhow::Result;

use std::ops::ControlFlow;

use super::{
    data::{event::EventsQueue, master_store::MasterStore},
    game_data::GameInstanceData,
    game_inputs, game_orchestrator, game_sync,
    game_timer::GameTimer,
};

use crate::websocket::WebSocketConnection;

pub struct GameInstance<'a> {
    client_conn: &'a mut WebSocketConnection,
    master_store: MasterStore,
    game_data: Box<GameInstanceData>,
    events_queue: EventsQueue,
}

impl<'a> GameInstance<'a> {
    pub fn new(
        client_conn: &'a mut WebSocketConnection,
        data: Box<GameInstanceData>,
        master_store: MasterStore,
    ) -> Self {
        GameInstance::<'a> {
            client_conn,
            master_store,
            game_data: data,

            events_queue: EventsQueue::new(),
        }
    }

    pub async fn run(mut self) -> Result<Box<GameInstanceData>> {
        game_sync::sync_init_game(self.client_conn, &mut self.game_data).await?;

        let mut game_timer = GameTimer::new();
        loop {
            game_orchestrator::reset_entities(&mut self.game_data).await;

            if let ControlFlow::Break(_) =
                game_inputs::handle_client_inputs(self.client_conn, &mut self.game_data).await
            {
                tracing::debug!("client disconnected...");
                break;
            }

            game_orchestrator::tick(
                &mut self.events_queue,
                &mut self.game_data,
                &self.master_store,
                game_timer.delta(),
            )
            .await?;

            if let Err(e) = game_sync::sync_update_game(self.client_conn, &mut self.game_data).await
            {
                tracing::warn!("failed to sync client: {}", e);
            }

            game_timer.wait_tick().await;
        }

        Ok(self.game_data)
    }
}
