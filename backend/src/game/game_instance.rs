use anyhow::Result;
use shared::messages::SessionId;

use super::{
    data::{event::EventsQueue, master_store::MasterStore},
    game_data::GameInstanceData,
    game_inputs, game_orchestrator, game_sync,
    game_timer::GameTimer,
};

use crate::{db::DbPool, game::systems::leaderboard_controller, websocket::WebSocketConnection};

pub struct GameInstance<'a> {
    client_conn: &'a mut WebSocketConnection,
    db_pool: DbPool,
    master_store: MasterStore,
    session_id: &'a SessionId,
    game_data: &'a mut GameInstanceData,
    events_queue: EventsQueue,
}

impl<'a> GameInstance<'a> {
    pub fn new(
        client_conn: &'a mut WebSocketConnection,
        session_id: &'a SessionId,
        game_data: &'a mut GameInstanceData,
        db_pool: DbPool,
        master_store: MasterStore,
    ) -> Self {
        GameInstance {
            client_conn,
            session_id,
            db_pool,
            master_store,
            game_data,

            events_queue: EventsQueue::new(),
        }
    }

    pub async fn run(mut self) -> Result<()> {
        game_sync::sync_init_game(self.client_conn, self.game_data).await?;

        let mut game_timer = GameTimer::new();
        loop {
            game_orchestrator::reset_entities(self.game_data).await;

            if game_inputs::handle_client_inputs(
                self.client_conn,
                self.game_data,
                &self.master_store,
            )
            .await
            .is_break()
            {
                break;
            }

            game_orchestrator::tick(
                &mut self.events_queue,
                self.game_data,
                &self.master_store,
                game_timer.delta(),
            )
            .await?;

            if let Err(e) = game_sync::sync_update_game(self.client_conn, self.game_data).await {
                tracing::warn!("failed to sync client: {}", e);
            }

            if game_timer.should_autosave() {
                let (db_pool, session_id, game_data) = (
                    self.db_pool.clone(),
                    *self.session_id,
                    self.game_data.clone(), // TODO: Do something else
                );
                tokio::spawn(async move {
                    leaderboard_controller::save_game_score(&db_pool, &session_id, &game_data).await
                });
            }

            game_timer.wait_tick().await;
        }

        tracing::debug!("game session '{}' ended ", self.session_id);
        Ok(())
    }
}
