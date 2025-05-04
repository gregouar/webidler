use anyhow::Result;

use tokio::task::yield_now;

use std::{
    ops::ControlFlow,
    time::{Duration, Instant},
};

use shared::{
    data::{
        item::ItemCategory,
        monster::{MonsterSpecs, MonsterState},
        player::{PlayerResources, PlayerSpecs, PlayerState},
        world::WorldState,
    },
    messages::{
        client::ClientMessage,
        server::{InitGameMessage, SyncGameStateMessage},
    },
};

use super::systems::{
    characters_updater, monsters_controller, monsters_updater, monsters_wave, player_controller,
    player_updater, skills_controller, skills_updater, weapon::update_weapon_specs,
};
use super::{data::DataInit, systems::player_controller::PlayerController, world::WorldBlueprint};

use crate::websocket::WebSocketConnection;

const LOOP_MIN_PERIOD: Duration = Duration::from_millis(100);

const PLAYER_RESPAWN_PERIOD: Duration = Duration::from_secs(5);

const MONSTER_WAVE_PERIOD: Duration = Duration::from_secs(1);
const WAVES_PER_AREA_LEVEL: u8 = 5;

pub struct GameInstance<'a> {
    client_conn: &'a mut WebSocketConnection,

    world_blueprint: WorldBlueprint,
    world_state: WorldState,

    player_specs: PlayerSpecsWrapper,
    player_state: PlayerState,
    player_resources: PlayerResources,
    player_controller: PlayerController,
    player_respawn_delay: Instant,

    need_to_sync_monster_specs: bool,
    monster_specs: Vec<MonsterSpecs>,
    monster_states: Vec<MonsterState>,
    monster_wave_delay: Instant,
}

struct PlayerSpecsWrapper {
    inner: PlayerSpecs,
    need_to_sync: bool,
}

impl PlayerSpecsWrapper {
    pub fn new(player_specs: PlayerSpecs) -> Self {
        Self {
            inner: player_specs,
            need_to_sync: true,
        }
    }

    pub fn mutate(&mut self) -> &mut PlayerSpecs {
        self.need_to_sync = true;
        &mut self.inner
    }

    pub fn read(&self) -> &PlayerSpecs {
        &self.inner
    }

    pub fn need_to_sync(&self) -> bool {
        self.need_to_sync
    }

    pub fn reset_sync(&mut self) {
        self.need_to_sync = false;
    }
}

impl<'a> GameInstance<'a> {
    pub fn new(
        client_conn: &'a mut WebSocketConnection,
        player_specs: PlayerSpecs,
        world_blueprint: WorldBlueprint,
    ) -> Self {
        GameInstance::<'a> {
            client_conn,

            world_state: WorldState::init(&world_blueprint.schema.specs),
            world_blueprint: world_blueprint,

            player_resources: PlayerResources {
                passive_points: 0,
                experience: 0.0,
                gold: 0.0,
            },
            player_state: PlayerState::init(&player_specs),
            player_controller: PlayerController::init(&player_specs),
            player_specs: PlayerSpecsWrapper::new(player_specs),
            player_respawn_delay: Instant::now(),

            need_to_sync_monster_specs: false,
            monster_specs: Vec::new(),
            monster_states: Vec::new(),
            monster_wave_delay: Instant::now(),
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        self.init_game().await?;

        let mut last_tick_time = Instant::now();
        let mut last_update_time = Instant::now();
        loop {
            if let ControlFlow::Break(_) = self.handle_client_events().await {
                break;
            }

            self.reset_entities().await;
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

        Ok(())
    }

    /// Handle client events, return whether the game should stop or continue
    async fn handle_client_events(&mut self) -> ControlFlow<(), ()> {
        // We limit the amount of events we handle in one loop
        for _ in 1..100 {
            match self.client_conn.poll_receive() {
                ControlFlow::Continue(Some(m)) => self.handle_client_message(m),
                ControlFlow::Continue(None) => return ControlFlow::Continue(()), // No more messages
                ControlFlow::Break(_) => return ControlFlow::Break(()), // Connection closed
            }
            yield_now().await;
        }
        ControlFlow::Continue(())
    }

    fn handle_client_message(&mut self, msg: ClientMessage) {
        match msg {
            ClientMessage::Heartbeat => {}
            ClientMessage::UseSkill(m) => {
                self.player_controller
                    .use_skills
                    .push(m.skill_index as usize);
            }
            ClientMessage::SetAutoSkill(m) => {
                self.player_controller
                    .auto_skills
                    .get_mut(m.skill_index as usize)
                    .map(|x| *x = m.auto_use);
            }
            ClientMessage::LevelUpSkill(m) => {
                if let Some(skill_specs) = self
                    .player_specs
                    .mutate()
                    .skill_specs
                    .get_mut(m.skill_index as usize)
                {
                    skills_controller::level_up_skill(skill_specs, &mut self.player_resources);
                }
            }
            ClientMessage::LevelUpPlayer => {
                player_controller::level_up(
                    &mut self.player_specs.mutate(),
                    &mut self.player_state,
                    &mut self.player_resources,
                );
            }
            ClientMessage::EquipItem(m) => player_controller::equip_item(
                self.player_specs.mutate(),
                &mut self.player_state,
                m.item_index,
            ),
            // Shouldn't receive other kind of messages:
            ClientMessage::Connect(_) => {
                tracing::warn!("received unexpected message: {:?}", msg)
            }
        }
    }

    async fn init_game(&mut self) -> Result<()> {
        // TODO: Remove, find better way to do this:
        if let Some(weapon) = self.player_specs.mutate().inventory.weapon_specs.as_mut() {
            let affix_effects = weapon.aggregate_effects();
            if let ItemCategory::Weapon(w) = &mut weapon.item_category {
                update_weapon_specs(w, affix_effects);
            }
        }
        for i in self.player_specs.mutate().inventory.bag.iter_mut() {
            let affix_effects = i.aggregate_effects();
            match &mut i.item_category {
                ItemCategory::Trinket => {}
                ItemCategory::Weapon(w) => update_weapon_specs(w, affix_effects),
            };
        }

        self.client_conn
            .send(
                &InitGameMessage {
                    world_specs: self.world_blueprint.schema.specs.clone(),
                    world_state: self.world_state.clone(),
                    player_specs: self.player_specs.read().clone(),
                    player_state: self.player_state.clone(),
                }
                .into(),
            )
            .await
    }

    async fn generate_monsters_wave(&mut self) -> Result<()> {
        self.world_state.waves_done += 1;
        if self.world_state.waves_done > WAVES_PER_AREA_LEVEL {
            self.world_state.waves_done = 1;
            self.world_state.area_level += 1;
        }

        self.monster_specs =
            monsters_wave::generate_monsters_wave_specs(&self.world_blueprint, &self.world_state)?;

        self.monster_states = self
            .monster_specs
            .iter()
            .map(|specs| MonsterState::init(specs))
            .collect();

        self.need_to_sync_monster_specs = true;

        Ok(())
    }

    fn respawn_player(&mut self) {
        self.monster_specs = Vec::new();
        self.monster_states = Vec::new();
        self.need_to_sync_monster_specs = true;

        self.player_state = PlayerState::init(self.player_specs.read());

        self.world_state.area_level = self.world_state.area_level.checked_sub(1).unwrap_or(1);
        self.world_state.waves_done = 0;
    }

    async fn reset_entities(&mut self) {
        // TODO: reset Player
        characters_updater::reset_character(&mut self.player_state.character_state);
        skills_updater::reset_skills(&mut self.player_state.skill_states);
        for monster_state in self.monster_states.iter_mut() {
            // TODO: reset Monster
            characters_updater::reset_character(&mut monster_state.character_state);
            skills_updater::reset_skills(&mut monster_state.skill_states);
        }
    }

    async fn control_entities(&mut self) -> Result<()> {
        if !self.player_state.character_state.is_alive {
            if self.player_respawn_delay.elapsed() > PLAYER_RESPAWN_PERIOD {
                self.respawn_player();
            }
        } else {
            self.player_respawn_delay = Instant::now();
            let mut monsters_still_alive: Vec<(&MonsterSpecs, &mut MonsterState)> = self
                .monster_specs
                .iter()
                .zip(self.monster_states.iter_mut())
                .filter(|(_, x)| x.character_state.is_alive)
                .collect();

            self.player_controller.control_player(
                self.player_specs.read(),
                &mut self.player_state,
                &mut monsters_still_alive,
            );
            self.player_controller.reset();

            // TODO: Where should I put this?
            for (monster_specs, _) in monsters_still_alive
                .iter()
                .filter(|(_, s)| s.character_state.just_died)
            {
                player_controller::reward_player(&mut self.player_resources, monster_specs);
            }

            if monsters_still_alive.is_empty() {
                if self.monster_wave_delay.elapsed() > MONSTER_WAVE_PERIOD {
                    self.generate_monsters_wave().await?;
                }
            } else {
                self.monster_wave_delay = Instant::now();
                monsters_controller::control_monsters(
                    &mut monsters_still_alive,
                    self.player_specs.read(),
                    &mut self.player_state,
                );
            }
        }

        Ok(())
    }

    async fn update_entities(&mut self, elapsed_time: Duration) {
        player_updater::update_player_state(
            elapsed_time,
            self.player_specs.read(),
            &mut self.player_state,
        );
        monsters_updater::update_monster_states(
            elapsed_time,
            &self.monster_specs,
            &mut self.monster_states,
        );
    }

    /// Send whole world state to client
    async fn sync_client(&mut self) -> Result<()> {
        self.client_conn
            .send(
                &SyncGameStateMessage {
                    world_state: self.world_state.clone(),
                    player_specs: if self.player_specs.need_to_sync() {
                        self.player_specs.reset_sync();
                        Some(self.player_specs.read().clone())
                    } else {
                        None
                    },
                    player_state: self.player_state.clone(),
                    player_resources: self.player_resources.clone(),
                    monster_specs: if self.need_to_sync_monster_specs {
                        self.need_to_sync_monster_specs = false;
                        Some(self.monster_specs.clone())
                    } else {
                        None
                    },
                    monster_states: self.monster_states.clone(),
                }
                .into(),
            )
            .await
    }
}
