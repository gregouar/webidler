use anyhow::Result;

use tokio::task::yield_now;

use std::{
    ops::ControlFlow,
    time::{Duration, Instant},
};

use rand::Rng;
use shared::{
    data::{
        CharacterPrototype, MonsterPrototype, MonsterState, PlayerPrototype, PlayerState,
        SkillPrototype,
    },
    messages::{
        client::ClientMessage,
        server::{InitGameMessage, SyncGameStateMessage},
    },
};

use crate::websocket::WebSocketConnection;

const LOOP_MIN_PERIOD: Duration = Duration::from_millis(100);
const MAX_MONSTERS: usize = 6;
const MONSTER_WAVE_PERIOD: Duration = Duration::from_secs(2);

pub struct GameInstance<'a> {
    client_conn: &'a mut WebSocketConnection,
    loop_counter: i32,
    // todo: map infos, player, monsters, etc
    player_prototype: PlayerPrototype,
    player_state: PlayerState,
    monster_prototypes: Vec<MonsterPrototype>,
    monster_states: Vec<MonsterState>,
    monster_wave_delay: Instant,
    need_to_sync_monster_prototypes: bool,
}

// TODO: split the logic in multiple systems

impl<'a> GameInstance<'a> {
    pub fn new(
        client_conn: &'a mut WebSocketConnection,
        player_prototype: PlayerPrototype,
    ) -> Self {
        GameInstance::<'a> {
            client_conn,
            loop_counter: 0,
            player_state: PlayerState::init(&player_prototype),
            player_prototype,
            monster_prototypes: Vec::new(),
            monster_states: Vec::new(),
            monster_wave_delay: Instant::now(),
            need_to_sync_monster_prototypes: false,
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        self.init_game().await?;

        let mut last_time = Instant::now();
        loop {
            self.loop_counter += 1;

            if let ControlFlow::Break(_) = self.handle_client_events().await {
                break;
            }

            self.update().await;

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
            ClientMessage::Heartbeat => {}
            ClientMessage::Test(m) => {
                tracing::info!("test: {:?}", m)
            }
            _ => {
                tracing::warn!("received unexpected message: {:?}", msg)
            }
        }
    }

    async fn init_game(&mut self) -> Result<()> {
        self.generate_monsters_wave().await;
        self.client_conn
            .send(
                &InitGameMessage {
                    player_prototype: self.player_prototype.clone(),
                    player_state: self.player_state.clone(),
                }
                .into(),
            )
            .await
    }

    async fn generate_monsters_wave(&mut self) {
        let mut rng = rand::rng();
        let n = rng.random_range(1..=MAX_MONSTERS);
        self.monster_prototypes = Vec::with_capacity(n);
        self.monster_states = Vec::with_capacity(n);
        for _ in 1..=n {
            let monster_type = rng.random_range(0..2);

            let prototype = match monster_type {
                0 => MonsterPrototype {
                    character_prototype: CharacterPrototype {
                        // identifier: i as u64,
                        name: String::from("batty"),
                        portrait: match rng.random_range(0..2) {
                            0 => String::from("monsters/bat.webp"),
                            _ => String::from("monsters/bat2.webp"),
                        },
                        max_health: 20,
                        skill_prototypes: vec![SkillPrototype {
                            name: String::from("Bite"),
                            icon: String::from("bite"), // TODO
                            cooldown: Duration::from_secs(3),
                            mana_cost: 0,
                            min_damages: 1,
                            max_damages: 3,
                        }],
                    },
                },
                _ => MonsterPrototype {
                    character_prototype: CharacterPrototype {
                        // identifier: i as u64,
                        name: String::from("ratty"),
                        portrait: String::from("monsters/rat.webp"),
                        max_health: 50,
                        skill_prototypes: vec![
                            SkillPrototype {
                                name: String::from("Vicious Bite"),
                                icon: String::from("bite"), // TODO
                                cooldown: Duration::from_secs(5),
                                mana_cost: 0,
                                min_damages: 2,
                                max_damages: 8,
                            },
                            SkillPrototype {
                                name: String::from("Scratch"),
                                icon: String::from("claw"), // TODO
                                cooldown: Duration::from_secs(3),
                                mana_cost: 0,
                                min_damages: 1,
                                max_damages: 3,
                            },
                        ],
                    },
                },
            };
            self.monster_states.push(MonsterState::init(&prototype));
            self.monster_prototypes.push(prototype);
        }
        self.need_to_sync_monster_prototypes = true;
    }

    async fn update(&mut self) {
        let mut still_alive: Vec<&mut MonsterState> = self
            .monster_states
            .iter_mut()
            .filter(|x| x.character_state.health > 0)
            .collect();

        if !still_alive.is_empty() {
            let mut rng = rand::rng();
            let i = rng.random_range(0..still_alive.len());
            still_alive.get_mut(i).map(|x| {
                x.character_state.health = x.character_state.health.checked_sub(5).unwrap_or(0);
                if x.character_state.health == 0 {
                    x.character_state.is_alive = false;
                }
            });
            self.monster_wave_delay = Instant::now();
        } else {
            if self.monster_wave_delay.elapsed() > MONSTER_WAVE_PERIOD {
                self.generate_monsters_wave().await;
            }
        }
    }

    /// Send whole world state to client
    async fn sync_client(&mut self) -> Result<()> {
        // TODO: Verify if need to update monster prototypes
        self.client_conn
            .send(
                &SyncGameStateMessage {
                    value: self.loop_counter,
                    player_state: self.player_state.clone(),
                    monster_prototypes: if self.need_to_sync_monster_prototypes {
                        self.need_to_sync_monster_prototypes = false;
                        Some(self.monster_prototypes.clone())
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
