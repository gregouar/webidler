use anyhow::Result;
use shared::messages::{SessionId, UserId};

use super::{
    data::{event::EventsQueue, master_store::MasterStore},
    game_data::GameInstanceData,
    game_inputs, game_orchestrator, game_sync,
    game_timer::GameTimer,
};

use crate::{
    db::{self, DbPool},
    game::systems::leaderboard_controller,
    websocket::WebSocketConnection,
};

pub struct GameInstance<'a> {
    client_conn: &'a mut WebSocketConnection,
    db_pool: DbPool,
    master_store: MasterStore,
    user_id: &'a UserId,
    session_id: &'a SessionId,
    game_data: &'a mut GameInstanceData,
    events_queue: EventsQueue,
}

impl<'a> GameInstance<'a> {
    pub fn new(
        client_conn: &'a mut WebSocketConnection,
        user_id: &'a UserId,
        session_id: &'a SessionId,
        game_data: &'a mut GameInstanceData,
        db_pool: DbPool,
        master_store: MasterStore,
    ) -> Self {
        GameInstance {
            client_conn,
            user_id,
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
                self.auto_save();
            }

            game_timer.wait_tick().await;
        }

        tracing::debug!("game session '{}' ended ", self.session_id);
        Ok(())
    }

    fn auto_save(&self) {
        let (db_pool, user_id, session_id, game_data) = (
            self.db_pool.clone(),
            self.user_id.clone(),
            *self.session_id,
            self.game_data.clone(), // TODO: Do something else, like only copy the necessary data
        );
        tokio::spawn(async move {
            leaderboard_controller::save_game_score(&db_pool, &session_id, &game_data)
                .await
                .unwrap_or_else(|e| {
                    tracing::error!("failed to save game score '{}': {}", user_id, e)
                });
            db::game_instances::save_game_instance_data(&db_pool, &user_id, game_data)
                .await
                .unwrap_or_else(|e| {
                    tracing::error!("failed to save game instance for user '{}': {}", user_id, e)
                });
        });
    }
}
