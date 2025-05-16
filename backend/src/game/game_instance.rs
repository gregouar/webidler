use anyhow::Result;

use tokio::task::yield_now;

use std::{
    ops::ControlFlow,
    time::{Duration, Instant},
};

use shared::{
    data::{
        monster::{MonsterSpecs, MonsterState},
        player::PlayerState,
    },
    messages::{
        client::ClientMessage,
        server::{ErrorMessage, ErrorType, InitGameMessage, SyncGameStateMessage},
    },
};

use super::{
    data::{master_store::MasterStore, DataInit},
    game_instance_data::GameInstanceData,
    systems::{loot_controller, loot_generator},
};
use super::{
    systems::{
        monsters_controller, monsters_updater, monsters_wave, player_controller, player_updater,
        skills_controller,
    },
    utils::LazySyncer,
};

use crate::websocket::WebSocketConnection;

const LOOP_MIN_PERIOD: Duration = Duration::from_millis(100);

const PLAYER_RESPAWN_PERIOD: Duration = Duration::from_secs(5);

const WAVES_PER_AREA_LEVEL: u8 = 5;

pub struct GameInstance<'a> {
    client_conn: &'a mut WebSocketConnection,
    master_store: MasterStore,
    data: Box<GameInstanceData>,
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
            data,
        }
    }

    pub async fn run(mut self) -> Result<Box<GameInstanceData>> {
        self.init_game().await?;

        let mut last_tick_time = Instant::now();
        let mut last_update_time = Instant::now();
        loop {
            self.reset_entities().await;

            if let ControlFlow::Break(_) = self.handle_client_events().await {
                break;
            }

            self.control_entities().await?;

            let elapsed_time = last_update_time.elapsed();
            last_update_time = Instant::now();
            self.update_entities(elapsed_time).await;

            if let Err(e) = self.sync_client().await {
                tracing::warn!("failed to sync client: {}", e);
            }

            // Wait for next tick
            let duration = last_tick_time.elapsed();
            if duration < LOOP_MIN_PERIOD {
                tokio::time::sleep(LOOP_MIN_PERIOD - duration).await;
            }
            last_tick_time = Instant::now();
        }

        Ok(self.data)
    }

    /// Handle client events, return whether the game should stop or continue
    async fn handle_client_events(&mut self) -> ControlFlow<(), ()> {
        // We limit the amount of events we handle in one loop
        for _ in 1..100 {
            match self.client_conn.poll_receive() {
                ControlFlow::Continue(Some(m)) => {
                    if let Some(error_message) = self.handle_client_message(m) {
                        if let Err(e) = self.client_conn.send(&error_message.into()).await {
                            tracing::warn!("failed to send error to client: {}", e)
                        }
                    }
                }
                ControlFlow::Continue(None) => return ControlFlow::Continue(()), // No more messages
                ControlFlow::Break(_) => return ControlFlow::Break(()), // Connection closed
            }
            yield_now().await;
        }
        ControlFlow::Continue(())
    }

    fn handle_client_message(&mut self, msg: ClientMessage) -> Option<ErrorMessage> {
        match msg {
            ClientMessage::Heartbeat => {}
            ClientMessage::UseSkill(m) => {
                self.data
                    .player_controller
                    .use_skills
                    .push(m.skill_index as usize);
            }
            ClientMessage::SetAutoSkill(m) => {
                self.data
                    .player_controller
                    .auto_skills
                    .get_mut(m.skill_index as usize)
                    .map(|x| *x = m.auto_use);
            }
            ClientMessage::LevelUpSkill(m) => {
                for _ in 0..m.amount {
                    if let Some(skill_specs) = self
                        .data
                        .player_specs
                        .mutate()
                        .skills_specs
                        .get_mut(m.skill_index as usize)
                    {
                        skills_controller::level_up_skill(
                            skill_specs,
                            &mut self.data.player_resources,
                        );
                    }
                }
            }
            ClientMessage::LevelUpPlayer(m) => {
                for _ in 0..m.amount {
                    player_controller::level_up(
                        &mut self.data.player_specs.mutate(),
                        &mut self.data.player_state,
                        &mut self.data.player_resources,
                    );
                }
            }
            ClientMessage::EquipItem(m) => player_controller::equip_item_from_bag(
                self.data.player_specs.mutate(),
                self.data.player_inventory.mutate(),
                &mut self.data.player_state,
                m.item_index,
            ),
            ClientMessage::SellItems(m) => {
                let mut item_indexes = m.item_indexes;
                item_indexes.sort_by_key(|&i| i);
                for &item_index in item_indexes.iter().rev() {
                    player_controller::sell_item(
                        self.data.player_inventory.mutate(),
                        &mut self.data.player_resources,
                        item_index,
                    )
                }
            }
            ClientMessage::PickupLoot(m) => {
                if !loot_controller::pickup_loot(
                    self.data.player_inventory.mutate(),
                    self.data.queued_loot.mutate(),
                    m.loot_identifier,
                ) {
                    return Some(ErrorMessage {
                        error_type: ErrorType::Game,
                        message: "Your bag is full!".to_string(),
                    });
                }
            }
            // Shouldn't receive other kind of messages:
            ClientMessage::Connect(_) => {
                tracing::warn!("received unexpected message: {:?}", msg);
                return Some(ErrorMessage {
                    error_type: ErrorType::Server,
                    message: "unexpected message received from client".to_string(),
                });
            }
        }
        None
    }

    async fn init_game(&mut self) -> Result<()> {
        self.data.reset_syncers();
        self.client_conn
            .send(
                &InitGameMessage {
                    world_specs: self.data.world_blueprint.specs.clone(),
                    world_state: self.data.world_state.clone(),
                    player_specs: self.data.player_specs.read().clone(),
                    player_state: self.data.player_state.clone(),
                }
                .into(),
            )
            .await
    }

    async fn generate_monsters_wave(&mut self) -> Result<()> {
        self.data.world_state.waves_done += 1;
        if self.data.world_state.waves_done > WAVES_PER_AREA_LEVEL {
            self.data.world_state.waves_done = 1;
            self.data.world_state.area_level += 1;
        }

        self.data.monster_specs = LazySyncer::new(monsters_wave::generate_monsters_wave_specs(
            &self.data.world_blueprint,
            &self.data.world_state,
            &self.master_store.monster_specs_store,
        )?);

        self.data.monster_states = self
            .data
            .monster_specs
            .read()
            .iter()
            .map(|specs| MonsterState::init(specs))
            .collect();

        Ok(())
    }

    fn respawn_player(&mut self) {
        self.data.monster_specs.mutate().clear();
        self.data.monster_states = Vec::new();

        self.data.player_state = PlayerState::init(self.data.player_specs.read());

        self.data.world_state.area_level = self
            .data
            .world_state
            .area_level
            .checked_sub(1)
            .unwrap_or(1)
            .max(1);
        self.data.world_state.waves_done = 0;
    }

    async fn reset_entities(&mut self) {
        player_updater::reset_player(&mut self.data.player_state);
        monsters_updater::reset_monsters(&mut self.data.monster_states);
    }

    async fn control_entities(&mut self) -> Result<()> {
        if !self.data.player_state.character_state.is_alive {
            if self.data.player_respawn_delay.elapsed() > PLAYER_RESPAWN_PERIOD {
                self.respawn_player();
            }
        } else {
            self.data.player_respawn_delay = Instant::now();
            let mut monsters_still_alive: Vec<(&MonsterSpecs, &mut MonsterState)> = self
                .data
                .monster_specs
                .read()
                .iter()
                .zip(self.data.monster_states.iter_mut())
                .filter(|(_, x)| x.character_state.is_alive)
                .collect();

            self.data.player_controller.control_player(
                self.data.player_specs.read(),
                &mut self.data.player_state,
                &mut monsters_still_alive,
            );
            self.data.player_controller.reset();

            // TODO: Where should I put this?
            for (monster_specs, _) in monsters_still_alive
                .iter()
                .filter(|(_, s)| s.character_state.just_died)
            {
                player_controller::reward_player(
                    &mut self.data.player_resources,
                    self.data.player_specs.read(),
                    monster_specs,
                );
            }

            if monsters_still_alive.is_empty() {
                if !self.data.looted && self.data.world_state.waves_done == WAVES_PER_AREA_LEVEL {
                    if let Some(item_specs) = loot_generator::generate_loot(
                        self.data.world_state.area_level,
                        &self.data.world_blueprint.loot_table,
                        &self.master_store.items_store,
                        &self.master_store.item_affixes_table,
                        &self.master_store.item_adjectives_table,
                        &self.master_store.item_nouns_table,
                    ) {
                        loot_controller::drop_loot(self.data.queued_loot.mutate(), item_specs);
                    }
                    self.data.looted = true;
                }

                if self.data.monster_wave_delay.elapsed()
                    > Duration::from_secs_f32(self.data.player_specs.read().movement_cooldown)
                {
                    self.generate_monsters_wave().await?;
                    self.data.looted = false;
                }
            } else {
                self.data.monster_wave_delay = Instant::now();
                monsters_controller::control_monsters(
                    &mut monsters_still_alive,
                    self.data.player_specs.read(),
                    &mut self.data.player_state,
                );
            }
        }

        Ok(())
    }

    async fn update_entities(&mut self, elapsed_time: Duration) {
        player_updater::update_player_state(
            elapsed_time,
            self.data.player_specs.read(),
            &mut self.data.player_state,
        );
        monsters_updater::update_monster_states(
            elapsed_time,
            self.data.monster_specs.read(),
            &mut self.data.monster_states,
        );
    }

    /// Send whole world state to client
    async fn sync_client(&mut self) -> Result<()> {
        self.client_conn
            .send(
                &SyncGameStateMessage {
                    world_state: self.data.world_state.clone(),
                    player_specs: self.data.player_specs.sync(),
                    player_inventory: self.data.player_inventory.sync(),
                    player_state: self.data.player_state.clone(),
                    player_resources: self.data.player_resources.clone(),
                    monster_specs: self.data.monster_specs.sync(),
                    monster_states: self.data.monster_states.clone(),
                    queued_loot: self.data.queued_loot.sync(),
                }
                .into(),
            )
            .await
    }
}
