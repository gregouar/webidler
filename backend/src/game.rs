use anyhow::Result;

use tokio::task::yield_now;

use std::{
    ops::ControlFlow,
    time::{Duration, Instant},
};

use rand::Rng;
use shared::{
    data::{CharacterPrototype, MonsterPrototype, MonsterState, PlayerPrototype, PlayerState},
    messages::{
        client::ClientMessage,
        server::{InitGameMessage, SyncGameStateMessage},
    },
};

use crate::websocket::WebSocketConnection;

const LOOP_MIN_PERIOD: Duration = Duration::from_millis(100);
const MAX_MONSTERS: usize = 6;

pub struct GameInstance<'a> {
    client_conn: &'a mut WebSocketConnection,
    loop_counter: i32,
    // todo: map infos, player, monsters, etc
    player_prototype: PlayerPrototype,
    player_state: PlayerState,
    monster_prototypes: Vec<MonsterPrototype>,
    monster_states: Vec<MonsterState>,
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

            self.sync_client().await?;

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
        for i in 1..=n {
            let monster_type = rng.random_range(0..2);

            let prototype = match monster_type {
                0 => MonsterPrototype {
                    character_prototype: CharacterPrototype {
                        identifier: i as u64,
                        name: String::from("batty"),
                        portrait: match rng.random_range(0..2) {
                            0 => String::from("monsters/bat.webp"),
                            _ => String::from("monsters/bat2.webp"),
                        },
                        max_health: 20,
                    },
                },
                _ => MonsterPrototype {
                    character_prototype: CharacterPrototype {
                        identifier: i as u64,
                        name: String::from("ratty"),
                        portrait: String::from("monsters/rat.webp"),
                        max_health: 50,
                    },
                },
            };
            self.monster_states.push(MonsterState::init(&prototype));
            self.monster_prototypes.push(prototype);
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
                    monsters: None,
                    monsters_state: self.monster_states.clone(),
                }
                .into(),
            )
            .await
    }
}
