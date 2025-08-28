use anyhow::Result;
use shared::data::user::UserCharacterId;

use super::{
    data::{event::EventsQueue, master_store::MasterStore},
    game_data::GameInstanceData,
    game_inputs, game_orchestrator, game_sync,
    game_timer::GameTimer,
};

use crate::{
    db::{self, DbPool},
    websocket::WebSocketConnection,
};

pub struct GameInstance<'a> {
    client_conn: &'a mut WebSocketConnection,
    db_pool: DbPool,
    master_store: MasterStore,
    character_id: &'a UserCharacterId,
    game_data: &'a mut GameInstanceData,
    events_queue: EventsQueue,
}

impl<'a> GameInstance<'a> {
    pub fn new(
        client_conn: &'a mut WebSocketConnection,
        character_id: &'a UserCharacterId,
        game_data: &'a mut GameInstanceData,
        db_pool: DbPool,
        master_store: MasterStore,
    ) -> Self {
        GameInstance {
            client_conn,
            character_id,
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

            if self.game_data.area_state.read().end_quest {
                break;
            }

            game_timer.wait_tick().await;
        }

        if self.game_data.area_state.read().end_quest {
            self.end_quest().await?;
        }

        tracing::debug!("game session '{}' ended ", self.character_id);
        Ok(())
    }

    fn auto_save(&self) {
        let (db_pool, character_id, game_data) = (
            self.db_pool.clone(),
            *self.character_id,
            self.game_data.clone(), // TODO: Do something else, like only copy the necessary data
        );
        tokio::spawn(async move {
            db::characters::update_character_progress(
                &db_pool,
                &character_id,
                &game_data.area_id,
                game_data.area_state.read().max_area_level_completed as i32,
                game_data.player_resources.read().gems,
                game_data.player_resources.read().shards,
            )
            .await
            .unwrap_or_else(|e| {
                tracing::error!(
                    "failed to save character progress '{}': {}",
                    character_id,
                    e
                )
            });

            db::game_instances::save_game_instance_data(&db_pool, &character_id, game_data)
                .await
                .unwrap_or_else(|e| {
                    tracing::error!(
                        "failed to save game instance for character '{}': {}",
                        character_id,
                        e
                    )
                });
        });
    }

    async fn end_quest(&self) -> Result<()> {
        db::characters_data::save_character_inventory(
            &self.db_pool,
            self.character_id,
            self.game_data.player_inventory.read(),
        )
        .await?;
        db::characters::update_character_progress(
            &self.db_pool,
            self.character_id,
            &self.game_data.area_id,
            self.game_data.area_state.read().max_area_level_completed as i32,
            self.game_data.player_resources.read().gems,
            self.game_data.player_resources.read().shards,
        )
        .await?;

        db::game_instances::delete_game_instance_data(&self.db_pool, self.character_id).await?;

        Ok(())
    }
}
