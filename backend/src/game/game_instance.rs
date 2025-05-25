use anyhow::Result;

use tokio::task::yield_now;

use std::{
    ops::ControlFlow,
    time::{Duration, Instant},
};

use shared::{
    data::{character::CharacterId, monster::MonsterState, player::PlayerState},
    messages::{
        client::ClientMessage,
        server::{ErrorMessage, ErrorType, InitGameMessage, SyncGameStateMessage},
    },
};

use super::{
    data::{
        event::{EventsQueue, GameEvent},
        master_store::MasterStore,
        DataInit,
    },
    game_instance_data::GameInstanceData,
    systems::{
        loot_controller, loot_generator, monsters_controller, monsters_updater, monsters_wave,
        passives_controller, player_controller, player_updater, skills_controller,
        world_controller,
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
            data,
            events_queue: EventsQueue::new(),
        }
    }

    pub async fn run(mut self) -> Result<Box<GameInstanceData>> {
        self.init_game().await?;

        let mut last_tick_time = Instant::now();
        let mut last_update_time = Instant::now();
        loop {
            self.reset_entities().await;

            if let ControlFlow::Break(_) = self.handle_client_events().await {
                tracing::debug!("client disconnected...");
                break;
            }

            if self.data.player_specs.need_to_sync()
                || self.data.player_inventory.need_to_sync()
                || self.data.passives_tree_state.need_to_sync()
            {
                player_controller::update_player_specs(
                    self.data.player_specs.mutate(),
                    self.data.player_inventory.read(),
                    &self.data.passives_tree_specs,
                    self.data.passives_tree_state.read(),
                );
            }

            self.control_entities().await?;

            self.resolve_events().await;

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
                if let Some(x) = self
                    .data
                    .player_controller
                    .auto_skills
                    .get_mut(m.skill_index as usize)
                {
                    *x = m.auto_use;
                }
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
                            self.data.player_resources.mutate(),
                        );
                    }
                }
            }
            ClientMessage::LevelUpPlayer(m) => {
                for _ in 0..m.amount {
                    player_controller::level_up(
                        self.data.player_specs.mutate(),
                        &mut self.data.player_state,
                        self.data.player_resources.mutate(),
                    );
                }
            }
            ClientMessage::EquipItem(m) => {
                if !player_controller::equip_item_from_bag(
                    self.data.player_specs.mutate(),
                    self.data.player_inventory.mutate(),
                    &mut self.data.player_state,
                    m.item_index,
                ) {
                    return Some(ErrorMessage {
                        error_type: ErrorType::Game,
                        message: "Not enough item slots available, please unequip first!"
                            .to_string(),
                    });
                }
            }
            ClientMessage::UnequipItem(m) => {
                if !player_controller::unequip_item_to_bag(
                    self.data.player_specs.mutate(),
                    self.data.player_inventory.mutate(),
                    &mut self.data.player_state,
                    m.item_slot,
                ) {
                    return Some(ErrorMessage {
                        error_type: ErrorType::Game,
                        message: "Your bag is full!".to_string(),
                    });
                }
            }
            ClientMessage::SellItems(m) => {
                let mut item_indexes = m.item_indexes;
                item_indexes.sort_by_key(|&i| i);
                for &item_index in item_indexes.iter().rev() {
                    player_controller::sell_item(
                        self.data.player_specs.read(),
                        self.data.player_inventory.mutate(),
                        self.data.player_resources.mutate(),
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
            ClientMessage::SetAutoProgress(m) => {
                self.data.world_state.mutate().auto_progress = m.value
            }
            ClientMessage::GoBack(m) => {
                let world_state = self.data.world_state.mutate();
                world_state.going_back += m.amount;
                world_state.auto_progress = false;
            }
            ClientMessage::PurchasePassive(m) => passives_controller::purchase_node(
                self.data.player_resources.mutate(),
                &self.data.passives_tree_specs,
                self.data.passives_tree_state.mutate(),
                m.node_id,
            ),
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
                    world_state: self.data.world_state.read().clone(),
                    passives_tree_specs: self.data.passives_tree_specs.clone(),
                    passives_tree_state: self.data.passives_tree_state.read().clone(),
                    player_specs: self.data.player_specs.read().clone(),
                    player_state: self.data.player_state.clone(),
                }
                .into(),
            )
            .await
    }

    async fn generate_monsters_wave(&mut self) -> Result<()> {
        let world_state = self.data.world_state.mutate();

        if world_state.going_back > 0 {
            let amount = world_state.going_back;
            world_controller::decrease_area_level(world_state, amount);
        }

        world_state.going_back = 0;
        world_state.waves_done += 1;

        if world_state.waves_done > WAVES_PER_AREA_LEVEL {
            world_state.waves_done = 1;
            self.data.game_stats.areas_completed += 1;
            if world_state.auto_progress {
                world_state.area_level += 1;
            }
        }

        self.data.game_stats.highest_area_level = self
            .data
            .game_stats
            .highest_area_level
            .max(world_state.area_level);

        self.data.monster_specs = LazySyncer::new(monsters_wave::generate_monsters_wave_specs(
            &self.data.world_blueprint,
            world_state,
            &self.master_store.monster_specs_store,
        )?);

        self.data.monster_states = self
            .data
            .monster_specs
            .read()
            .iter()
            .map(MonsterState::init)
            .collect();

        Ok(())
    }

    fn respawn_player(&mut self) {
        self.data.monster_specs.mutate().clear();
        self.data.monster_states = Vec::new();

        self.data.player_state = PlayerState::init(self.data.player_specs.read());

        world_controller::decrease_area_level(self.data.world_state.mutate(), 1);
    }

    async fn reset_entities(&mut self) {
        player_updater::reset_player(&mut self.data.player_state);
        monsters_updater::reset_monsters(&mut self.data.monster_states);
    }

    async fn control_entities(&mut self) -> Result<()> {
        if !self.data.player_state.character_state.is_alive {
            if self.data.player_respawn_delay.elapsed() > PLAYER_RESPAWN_PERIOD {
                self.data.game_stats.player_deaths += 1;
                self.respawn_player();
            }
        } else {
            self.data.player_respawn_delay = Instant::now();
            // let mut monsters_still_alive: Vec<(&MonsterSpecs, &mut MonsterState)> = self
            //     .data
            //     .monster_specs
            //     .read()
            //     .iter()
            //     .zip(self.data.monster_states.iter_mut())
            //     .filter(|(_, x)| x.character_state.is_alive)
            //     .collect();

            let mut monsters_still_alive: Vec<_> = self
                .data
                .monster_specs
                .read()
                .iter()
                .zip(self.data.monster_states.iter_mut())
                .enumerate()
                .filter(|(_, (_, m))| m.character_state.is_alive)
                .map(|(i, (x, y))| {
                    (
                        CharacterId::Monster(i),
                        (&x.character_specs, &mut y.character_state),
                    )
                })
                .collect();

            self.data.player_controller.control_player(
                &mut self.events_queue,
                self.data.player_specs.read(),
                &mut self.data.player_state,
                &mut monsters_still_alive,
            );
            self.data.player_controller.reset();

            if monsters_still_alive.is_empty() || self.data.world_state.read().going_back > 0 {
                if self.data.world_state.read().going_back == 0
                    && !self.data.looted
                    && self.data.world_state.read().waves_done == WAVES_PER_AREA_LEVEL
                {
                    if let Some(item_specs) = loot_generator::generate_loot(
                        self.data.world_state.read().area_level,
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
                    &mut self.events_queue,
                    self.data.monster_specs.read(),
                    &mut self.data.monster_states,
                    self.data.player_specs.read(),
                    &mut self.data.player_state,
                );
            }
        }

        Ok(())
    }

    async fn resolve_events(&mut self) {
        for event in self.events_queue.consume_events() {
            // TODO
            match event {
                GameEvent::Hit(damage_event) => todo!(),
                GameEvent::CriticalStrike(damage_event) => todo!(),
                GameEvent::Block(damage_event) => todo!(),
                GameEvent::Kill { target } => {
                    if let CharacterId::Monster(monster_index) = target {
                        self.data.game_stats.monsters_killed += 1;
                        if let Some(monster_specs) =
                            self.data.monster_specs.read().get(monster_index as usize)
                        {
                            player_controller::reward_player(
                                self.data.player_resources.mutate(),
                                self.data.player_specs.read(),
                                monster_specs,
                            );
                        }
                    }
                }
            }
        }
    }

    async fn update_entities(&mut self, elapsed_time: Duration) {
        self.data.game_stats.elapsed_time += elapsed_time;
        player_updater::update_player_state(
            &mut self.events_queue,
            elapsed_time,
            self.data.player_specs.read(),
            &mut self.data.player_state,
        );
        monsters_updater::update_monster_states(
            &mut self.events_queue,
            elapsed_time,
            self.data.monster_specs.read(),
            &mut self.data.monster_states,
        );
    }

    /// Send whole game state to client
    async fn sync_client(&mut self) -> Result<()> {
        self.client_conn
            .send(
                &SyncGameStateMessage {
                    world_state: self.data.world_state.sync(),
                    passives_tree_state: self.data.passives_tree_state.sync(),
                    player_specs: self.data.player_specs.sync(),
                    player_inventory: self.data.player_inventory.sync(),
                    player_state: self.data.player_state.clone(),
                    player_resources: self.data.player_resources.sync(),
                    monster_specs: self.data.monster_specs.sync(),
                    monster_states: self.data.monster_states.clone(),
                    queued_loot: self.data.queued_loot.sync(),
                    game_stats: self.data.game_stats.clone(),
                }
                .into(),
            )
            .await
    }
}
