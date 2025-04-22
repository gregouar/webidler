use anyhow::Result;

use tokio::task::yield_now;

use std::{
    ops::ControlFlow,
    time::{Duration, Instant},
};

use rand::Rng;
use shared::{
    data::{
        CharacterSpecs, MonsterSpecs, MonsterState, PlayerSpecs, PlayerState, SkillSpecs,
        WorldSpecs, WorldState,
    },
    messages::{
        client::ClientMessage,
        server::{InitGameMessage, SyncGameStateMessage},
    },
};

use super::systems::{character_controller, monsters_controller, monsters_updater, player_updater};
use super::{data::DataInit, systems::player_controller::PlayerController, world::WorldBlueprint};

use crate::websocket::WebSocketConnection;

const LOOP_MIN_PERIOD: Duration = Duration::from_millis(100);
const MAX_MONSTERS: usize = 6;
const MONSTER_WAVE_PERIOD: Duration = Duration::from_secs(1);

pub struct GameInstance<'a> {
    client_conn: &'a mut WebSocketConnection,
    // todo: map infos, player, monsters, etc
    world_blueprint: WorldBlueprint,
    world_specs: WorldSpecs,
    world_state: WorldState,
    player_specs: PlayerSpecs,
    player_state: PlayerState,
    player_controller: PlayerController,
    monster_specs: Vec<MonsterSpecs>,
    monster_states: Vec<MonsterState>,
    monster_wave_delay: Instant,
    need_to_sync_monster_specs: bool,
}

// TODO: split the logic in multiple systems

impl<'a> GameInstance<'a> {
    pub fn new(
        client_conn: &'a mut WebSocketConnection,
        player_specs: PlayerSpecs,
        world_blueprint: WorldBlueprint,
    ) -> Self {
        let world_specs = world_blueprint.schema.specs.clone();
        GameInstance::<'a> {
            client_conn,
            world_blueprint: world_blueprint,
            world_state: WorldState::init(&world_specs),
            world_specs: world_specs,
            player_state: PlayerState::init(&player_specs),
            player_controller: PlayerController::init(&player_specs),
            player_specs,
            monster_specs: Vec::new(),
            monster_states: Vec::new(),
            monster_wave_delay: Instant::now(),
            need_to_sync_monster_specs: false,
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        self.init_game().await?;

        let mut last_time = Instant::now();
        let mut last_update_time = Instant::now();
        loop {
            if let ControlFlow::Break(_) = self.handle_client_events().await {
                break;
            }

            self.reset_entities().await;
            self.control_entities().await;

            let elapsed_time = last_update_time.elapsed();
            last_update_time = Instant::now();
            self.update_entities(elapsed_time).await;

            if let Err(e) = self.sync_client().await {
                tracing::warn!("failed to sync client: {}", e);
            }

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
            ClientMessage::Connect(_) => {
                tracing::warn!("received unexpected message: {:?}", msg)
            }
        }
    }

    async fn init_game(&mut self) -> Result<()> {
        self.client_conn
            .send(
                &InitGameMessage {
                    world_specs: self.world_specs.clone(),
                    world_state: self.world_state.clone(),
                    player_specs: self.player_specs.clone(),
                    player_state: self.player_state.clone(),
                }
                .into(),
            )
            .await
    }

    async fn generate_monsters_wave(&mut self) {
        let mut rng = rand::rng();
        let n = rng.random_range(1..=MAX_MONSTERS);
        self.monster_specs = Vec::with_capacity(n);
        self.monster_states = Vec::with_capacity(n);
        for _ in 1..=n {
            let monster_type = rng.random_range(0..2);

            let specs = match monster_type {
                0 => MonsterSpecs {
                    character_specs: CharacterSpecs {
                        // identifier: i as u64,
                        name: String::from("Batty"),
                        portrait: match rng.random_range(0..2) {
                            0 => String::from("monsters/bat.webp"),
                            _ => String::from("monsters/bat2.webp"),
                        },
                        max_health: 10.0,
                        health_regen: 0.0,
                        skill_specs: vec![SkillSpecs {
                            name: String::from("Bite"),
                            icon: String::from("icons/bite.svg"), // TODO
                            cooldown: 3.0,
                            mana_cost: 0.0,
                            min_damages: 1.0,
                            max_damages: 3.0,
                        }],
                    },
                    max_initiative: 0.5,
                },
                _ => MonsterSpecs {
                    character_specs: CharacterSpecs {
                        // identifier: i as u64,
                        name: String::from("Ratty"),
                        portrait: String::from("monsters/rat.webp"),
                        max_health: 20.0,
                        health_regen: 0.0,
                        skill_specs: vec![
                            SkillSpecs {
                                name: String::from("Vicious Bite"),
                                icon: String::from("icons/bite.svg"),
                                cooldown: 5.0,
                                mana_cost: 0.0,
                                min_damages: 2.0,
                                max_damages: 8.0,
                            },
                            SkillSpecs {
                                name: String::from("Scratch"),
                                icon: String::from("icons/claw.svg"),
                                cooldown: 3.0,
                                mana_cost: 0.0,
                                min_damages: 1.0,
                                max_damages: 3.0,
                            },
                        ],
                    },
                    max_initiative: 1.0,
                },
            };
            self.monster_states.push(MonsterState::init(&specs));
            self.monster_specs.push(specs);
        }
        self.need_to_sync_monster_specs = true;
    }

    async fn reset_entities(&mut self) {
        character_controller::reset_character(&mut self.player_state.character_state);
        for monster_state in self.monster_states.iter_mut() {
            character_controller::reset_character(&mut monster_state.character_state);
        }
    }

    async fn control_entities(&mut self) {
        let mut monsters_still_alive: Vec<(&mut MonsterState, &MonsterSpecs)> = self
            .monster_states
            .iter_mut()
            .zip(self.monster_specs.iter())
            .filter(|(x, _)| x.character_state.is_alive)
            .collect();

        self.player_controller.control_player(
            &self.player_specs,
            &mut self.player_state,
            &mut monsters_still_alive,
        );
        self.player_controller.reset();

        if monsters_still_alive.is_empty() {
            if self.monster_wave_delay.elapsed() > MONSTER_WAVE_PERIOD {
                self.generate_monsters_wave().await;
            }
        } else {
            self.monster_wave_delay = Instant::now();
            monsters_controller::control_monsters(
                &mut monsters_still_alive,
                &self.player_specs,
                &mut self.player_state,
            );
        }
    }

    async fn update_entities(&mut self, elapsed_time: Duration) {
        player_updater::update_player_state(
            elapsed_time,
            &self.player_specs,
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
                    player_state: self.player_state.clone(),
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
