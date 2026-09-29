use std::collections::HashMap;

use anyhow::Result;

use chrono::Utc;
use shared::{
    computations,
    constants::{self, RUSH_MODE_SPEED_MULTIPLIER},
    data::{
        realms::Realm,
        user::{UserCharacterId, UserId},
    },
    messages::server::{AchievementsUnlockedMessage, ErrorMessage, ErrorType, ServerMessage},
};

use crate::{
    app_state::{MasterStore, SessionsStore},
    db::{self, DbPool},
    game::{
        data::event::EventsQueue,
        game_data::GameInstanceData,
        game_inputs, game_orchestrator, game_sync,
        game_timer::GameTimer,
        systems::achievements_controller::{self, AchievementContext},
    },
    integration::chat::ChatIntegration,
    websocket::WebSocketConnection,
};

pub struct GameInstance<'a> {
    client_conn: &'a mut WebSocketConnection,
    db_pool: DbPool,
    chat_integration: ChatIntegration,
    master_store: MasterStore,
    sessions_store: SessionsStore,
    user_id: UserId,
    character_id: &'a UserCharacterId,
    game_data: &'a mut GameInstanceData,
    events_queue: EventsQueue,
}

impl<'a> GameInstance<'a> {
    pub fn new(
        client_conn: &'a mut WebSocketConnection,
        user_id: UserId,
        character_id: &'a UserCharacterId,
        game_data: &'a mut GameInstanceData,
        db_pool: DbPool,
        chat_integration: ChatIntegration,
        master_store: MasterStore,
        sessions_store: SessionsStore,
    ) -> Self {
        GameInstance {
            client_conn,
            user_id,
            character_id,
            db_pool,
            chat_integration,
            master_store,
            sessions_store,
            game_data,

            events_queue: EventsQueue::new(),
        }
    }

    pub async fn run(mut self) -> Result<()> {
        let passives_tree_build =
            db::characters_builds::load_character_build(&self.db_pool, self.character_id)
                .await
                .ok()
                .flatten()
                .unwrap_or_default();
        let character_cosmetics =
            db::characters::read_character_cosmetics(&self.db_pool, &self.user_id)
                .await
                .ok()
                .flatten()
                .unwrap_or_default();
        let user_unlocks = db::user_unlocks::load_user_unlocks(&self.db_pool, &self.user_id)
            .await
            .unwrap_or_default();
        let player_pets =
            db::characters_data::load_character_pets(&self.db_pool, self.character_id)
                .await
                .unwrap_or_default();

        game_sync::sync_init_game(
            self.client_conn,
            self.character_id,
            self.game_data,
            passives_tree_build,
            character_cosmetics,
            user_unlocks,
            player_pets,
        )
        .await?;

        let mut game_timer = GameTimer::new();
        loop {
            if !self.game_data.end_grind {
                game_orchestrator::reset_entities(self.game_data).await;
            }

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

            if self.game_data.terminate_grind {
                break;
            }

            let elapsed_time = game_timer.delta();
            let tick_multiplier = if self.game_data.area_state.read().rush_mode {
                RUSH_MODE_SPEED_MULTIPLIER
            } else {
                1
            };
            for _ in 0..tick_multiplier {
                game_orchestrator::tick(
                    &mut self.events_queue,
                    self.game_data,
                    &self.master_store,
                    elapsed_time,
                )
                .await?;
            }

            match game_sync::sync_update_game(self.client_conn, self.game_data).await {
                Ok(true) => {}
                Ok(false) => tracing::debug!("skipping sync update while previous send is pending"),
                Err(e) => {
                    tracing::warn!("failed to sync client: {}", e);
                    break;
                }
            }

            if game_timer.should_autosave() {
                self.auto_save();
            }
            if game_timer.should_check_achievements() {
                self.check_achievements().await.unwrap_or_else(|error| {
                    tracing::error!(
                        "failed to update in-game achievements for user '{}': {error}",
                        self.user_id
                    )
                });
            }

            if self
                .sessions_store
                .sessions_stealing
                .remove(self.character_id)
                .is_some()
            {
                self.client_conn
                    .send(
                        &ErrorMessage {
                            error_type: ErrorType::Server,
                            message: "kicked out of game session".into(),
                            must_disconnect: true,
                        }
                        .into(),
                    )
                    .await
                    .unwrap_or_else(|_| tracing::warn!("failed to send disconnection message"));
                tracing::debug!("game session '{}' stolen ", self.character_id);
                return Ok(());
            }

            game_timer.wait_tick().await;
        }

        if self.game_data.terminate_grind {
            self.terminate_grind().await?;
        }

        self.client_conn
            .send(&ServerMessage::Disconnect)
            .await
            .unwrap_or_else(|_| tracing::warn!("failed to send disconnection message"));

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
            auto_save_impl(db_pool, character_id, game_data)
                .await
                .unwrap_or_else(|e| {
                    tracing::error!(
                        "failed to save character progress '{}': {}",
                        character_id,
                        e
                    )
                });
        });
    }

    async fn check_achievements(&mut self) -> Result<()> {
        let area_levels = HashMap::from([(
            self.game_data.area_id.clone(),
            self.game_data.area_state.read().max_area_level_ever,
        )]);
        let context = AchievementContext {
            area_levels: &area_levels,
            power_level: self.game_data.player_base_specs.read().max_area_level,
            player_level: self.game_data.player_base_specs.read().level,
            skill_masteries: &self.game_data.player_base_specs.read().skill_masteries,
            ascension: &self.game_data.passives_tree_state.read().ascension,
            inventory: self.game_data.player_inventory.read(),
        };

        let new_achievements = achievements_controller::check_achievements(
            &self.master_store,
            &self.game_data.user_achievements,
            &context,
        );

        if new_achievements.is_empty() {
            return Ok(());
        }

        for achievement_id in new_achievements.iter() {
            self.game_data
                .user_achievements
                .insert(achievement_id.clone(), Utc::now());
        }

        // let mut tx = self.db_pool.begin().await?;
        // let achievement_ids = achievements_controller::update_achievements(
        //     &mut tx,
        //     &self.master_store,
        //     self.user_id,
        //     &mut self.game_data.user_achievements,
        //     &context,
        // )
        // .await?;
        // tx.commit().await?;

        self.client_conn
            .send(
                &AchievementsUnlockedMessage {
                    achievement_ids: new_achievements.clone(),
                }
                .into(),
            )
            .await?;

        let (db_pool, master_store, user_id) = (
            self.db_pool.clone(),
            self.master_store.clone(),
            self.user_id,
        );
        tokio::spawn(async move {
            update_achievements_impl(db_pool, &master_store, user_id, new_achievements)
                .await
                .unwrap_or_else(|e| {
                    tracing::error!("failed to  update achievements '{}': {}", user_id, e)
                });
        });

        Ok(())
    }

    async fn terminate_grind(&self) -> Result<()> {
        let mut tx = self.db_pool.begin().await?;

        if !self.game_data.area_specs.training {
            db::characters_data::save_character_inventory(
                &mut *tx,
                self.character_id,
                self.game_data.player_inventory.read(),
            )
            .await?;

            let mut player_skill_masteries = self
                .game_data
                .player_base_specs
                .read()
                .skill_masteries
                .clone();
            for (skill_id, experience) in self
                .game_data
                .player_resources
                .read()
                .skill_masteries_experience
                .iter()
            {
                player_skill_masteries
                    .masteries
                    .entry(skill_id.clone())
                    .or_default()
                    .experience += experience;
            }
            db::characters_data::save_character_skill_masteries(
                &mut *tx,
                self.character_id,
                &player_skill_masteries,
            )
            .await?;

            let character_resources = db::characters::update_character_resources(
                &mut *tx,
                self.character_id,
                self.game_data.player_resources.read().gems,
                self.game_data.player_resources.read().shards,
                self.game_data.player_resources.read().gold_total
                    * computations::exponential(
                        *self.game_data.area_specs.item_level_modifier
                            + *self.game_data.area_specs.power_level,
                        constants::MONSTER_REWARD_INCREASE_FACTOR,
                    ),
                self.game_data.game_stats.elapsed_time.as_secs_f64(),
            )
            .await?;

            db::characters::set_character_stamina(
                &mut *tx,
                self.character_id,
                computations::stamina_spill(self.game_data.player_stamina).as_secs_f64(),
            )
            .await?;

            db::characters::update_character_max_area_level(
                &mut tx,
                self.character_id,
                self.game_data.player_base_specs.read().max_area_level as i32,
            )
            .await?;

            let max_area_level_ever = self.game_data.area_state.read().max_area_level_ever as i32;
            let max_power_shard_level_ever =
                self.game_data.area_state.read().max_power_shard_level_ever as i32;
            let quest_completed = self.game_data.quest_completed
                || self
                    .game_data
                    .area_specs
                    .quest
                    .as_ref()
                    .map(|quest| {
                        self.game_data.area_state.read().max_area_level >= quest.area_level
                    })
                    .unwrap_or_default();
            db::characters::update_character_area_progress(
                &mut tx,
                self.character_id,
                &self.game_data.area_id,
                max_area_level_ever,
                max_power_shard_level_ever,
                quest_completed,
            )
            .await?;

            if self.game_data.area_state.read().max_area_level > 0 {
                let realm_id = self.game_data.realm.realm_id();
                let realm_label = match self.game_data.realm {
                    Realm::Standard => "",
                    Realm::StandardSSF => " [SSF]",
                    Realm::Legacy => " [Legacy]",
                    Realm::LegacySSF => " [Legacy SSF]",
                };
                if let Err(err) = db::game_stats::save_game_stats(
                    &mut *tx,
                    self.character_id,
                    &realm_id,
                    self.game_data,
                )
                .await
                {
                    tracing::error!("failed to save game stats '{}': {}", self.character_id, err);
                }

                let power_level = self.game_data.player_base_specs.read().max_area_level;
                match db::leaderboard::update_leaderboard(
                    &mut tx,
                    self.character_id,
                    &realm_id,
                    constants::POWER_LEVEL_LEADERBOARD_AREA_ID,
                    power_level as i32,
                    character_resources.played_time_seconds,
                )
                .await
                {
                    Ok(true) => {
                        if let Err(err) = self
                            .chat_integration
                            .broadcast_message(
                                format!(
                                    "'{}'{} is the first to reach Power Level {}!",
                                    self.game_data
                                        .player_base_specs
                                        .read()
                                        .character_static
                                        .name,
                                    realm_label,
                                    power_level,
                                ),
                                None,
                            )
                            .await
                        {
                            tracing::error!("failed to broadcast power level highscore: {}", err);
                        }
                    }
                    Err(err) => {
                        tracing::error!(
                            "failed to update power level leaderboard '{}': {}",
                            self.character_id,
                            err
                        );
                    }
                    _ => {}
                }

                match db::leaderboard::update_leaderboard(
                    &mut tx,
                    self.character_id,
                    &realm_id,
                    &self.game_data.area_id,
                    self.game_data.area_state.read().max_area_level as i32,
                    self.game_data
                        .game_stats
                        .elapsed_time_at_max_level
                        .as_secs_f64(),
                )
                .await
                {
                    Ok(true) => {
                        if let Err(err) = self
                            .chat_integration
                            .broadcast_message(
                                format!(
                                    "'{}'{} is the first to beat Area Level {:0} in '{}'!",
                                    self.game_data
                                        .player_base_specs
                                        .read()
                                        .character_static
                                        .name,
                                    realm_label,
                                    self.game_data.area_state.read().max_area_level,
                                    self.game_data.area_specs.name,
                                ),
                                None,
                            )
                            .await
                        {
                            tracing::error!("failed to broadcast highscore: {}", err);
                        }
                    }
                    Err(err) => {
                        tracing::error!(
                            "failed to update leaderboard '{}': {}",
                            self.character_id,
                            err
                        );
                    }
                    _ => {}
                }
            }
        }

        db::game_instances::delete_game_instance_data(&mut *tx, self.character_id).await?;

        tx.commit().await?;

        Ok(())
    }
}

async fn auto_save_impl(
    db_pool: DbPool,
    character_id: UserCharacterId,
    game_data: GameInstanceData,
) -> Result<()> {
    let mut tx = db_pool.begin().await?;

    // db::characters::update_character_progress(
    //     &mut tx,
    //     &character_id,
    //     &game_data.area_id,
    //     game_data.area_state.read().max_area_level_completed as i32,
    // )
    // .await?;
    db::game_instances::save_game_instance_data(&mut *tx, &character_id, game_data).await?;

    tx.commit().await?;

    Ok(())
}

async fn update_achievements_impl(
    db_pool: DbPool,
    master_store: &MasterStore,
    user_id: UserId,
    new_achievements: Vec<String>,
) -> Result<()> {
    let mut tx = db_pool.begin().await?;
    achievements_controller::unlock_achievements(
        &mut tx,
        &master_store,
        user_id,
        &new_achievements,
    )
    .await?;
    tx.commit().await?;

    Ok(())
}
