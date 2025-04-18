use anyhow::Result;

use tokio::task::yield_now;

use std::{
    ops::ControlFlow,
    time::{Duration, Instant},
};

use rand::Rng;
use shared::{
    data::{
        CharacterPrototype, CharacterState, MonsterPrototype, MonsterState, PlayerPrototype,
        PlayerState, SkillPrototype, SkillState,
    },
    messages::{
        client::ClientMessage,
        server::{InitGameMessage, SyncGameStateMessage},
    },
};

use super::data::DataInit;

use crate::websocket::WebSocketConnection;

const LOOP_MIN_PERIOD: Duration = Duration::from_millis(100);
const MAX_MONSTERS: usize = 6;
const MONSTER_WAVE_PERIOD: Duration = Duration::from_secs(2);

pub struct GameInstance<'a> {
    client_conn: &'a mut WebSocketConnection,
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
                        max_health: 10.0,
                        health_regen: 0.0,
                        skill_prototypes: vec![SkillPrototype {
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
                _ => MonsterPrototype {
                    character_prototype: CharacterPrototype {
                        // identifier: i as u64,
                        name: String::from("ratty"),
                        portrait: String::from("monsters/rat.webp"),
                        max_health: 20.0,
                        health_regen: 0.0,
                        skill_prototypes: vec![
                            SkillPrototype {
                                name: String::from("Vicious Bite"),
                                icon: String::from("icons/bite.svg"),
                                cooldown: 5.0,
                                mana_cost: 0.0,
                                min_damages: 2.0,
                                max_damages: 8.0,
                            },
                            SkillPrototype {
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
            self.monster_states.push(MonsterState::init(&prototype));
            self.monster_prototypes.push(prototype);
        }
        self.need_to_sync_monster_prototypes = true;
    }

    async fn reset_entities(&mut self) {
        reset_character(&mut self.player_state.character_state);
        for monster_state in self.monster_states.iter_mut() {
            reset_character(&mut monster_state.character_state);
        }
    }

    async fn control_entities(&mut self) {
        let mut monsters_still_alive: Vec<(&mut MonsterState, &MonsterPrototype)> = self
            .monster_states
            .iter_mut()
            .zip(self.monster_prototypes.iter())
            .filter(|(x, _)| x.character_state.is_alive)
            .collect();

        control_player(
            &self.player_prototype,
            &mut self.player_state,
            &mut monsters_still_alive,
        );

        if monsters_still_alive.is_empty() {
            if self.monster_wave_delay.elapsed() > MONSTER_WAVE_PERIOD {
                self.generate_monsters_wave().await;
            }
        } else {
            self.monster_wave_delay = Instant::now();
            control_monsters(
                &mut monsters_still_alive,
                &self.player_prototype,
                &mut self.player_state,
            );
        }
    }

    async fn update_entities(&mut self, elapsed_time: Duration) {
        update_player_state(elapsed_time, &self.player_prototype, &mut self.player_state);
        update_monster_states(
            elapsed_time,
            &self.monster_prototypes,
            &mut self.monster_states,
        );
    }

    /// Send whole world state to client
    async fn sync_client(&mut self) -> Result<()> {
        self.client_conn
            .send(
                &SyncGameStateMessage {
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

fn reset_character(character_state: &mut CharacterState) {
    character_state.just_hurt = false;
    for skill_sate in character_state.skill_states.iter_mut() {
        reset_skill(skill_sate)
    }
}

fn reset_skill(skill_state: &mut SkillState) {
    skill_state.just_triggered = false;
}

fn control_player(
    player_prototype: &PlayerPrototype,
    player_state: &mut PlayerState,
    monsters: &mut Vec<(&mut MonsterState, &MonsterPrototype)>,
) {
    if !player_state.character_state.is_alive {
        return;
    }
    let mut rng = rand::rng();

    if !monsters.is_empty() {
        for (skill_prototype, skill_state) in player_prototype
            .character_prototype
            .skill_prototypes
            .iter()
            .zip(player_state.character_state.skill_states.iter_mut())
        {
            if !skill_state.is_ready || skill_prototype.mana_cost > player_state.mana {
                continue;
            }

            let j = rng.random_range(0..monsters.len());
            if let Some((target_state, target_prototype)) = monsters.get_mut(j).as_deref_mut() {
                player_state.mana -= skill_prototype.mana_cost;
                use_skill(
                    skill_prototype,
                    skill_state,
                    &mut target_state.character_state,
                    &target_prototype.character_prototype,
                );
            }
        }
    }
}

fn control_monsters(
    monsters: &mut Vec<(&mut MonsterState, &MonsterPrototype)>,
    player_prototype: &PlayerPrototype,
    player_state: &mut PlayerState,
) {
    for (monster_state, monster_prototype) in monsters
        .iter_mut()
        .filter(|(m, _)| m.character_state.is_alive && m.initiative == 0.0)
    {
        for (skill_prototype, skill_state) in monster_prototype
            .character_prototype
            .skill_prototypes
            .iter()
            .zip(monster_state.character_state.skill_states.iter_mut())
            .filter(|(_, s)| s.is_ready)
        {
            use_skill(
                &skill_prototype,
                skill_state,
                &mut player_state.character_state,
                &player_prototype.character_prototype,
            );
        }
    }
}

fn use_skill(
    skill_prototype: &SkillPrototype,
    skill_state: &mut SkillState,
    target_state: &mut CharacterState,
    target_prototype: &CharacterPrototype,
) {
    let mut rng = rand::rng();

    skill_state.just_triggered = true;
    skill_state.is_ready = false;
    skill_state.elapsed_cooldown = 0.0;

    if skill_prototype.max_damages >= skill_prototype.min_damages {
        damage_character(
            rng.random_range(skill_prototype.min_damages..=skill_prototype.max_damages),
            target_state,
            target_prototype,
        );
    }
}

fn damage_character(
    damages: f64,
    target_state: &mut CharacterState,
    target_prototype: &CharacterPrototype,
) {
    let _ = target_prototype;
    target_state.health = (target_state.health - damages).max(0.0);
    if target_state.health == 0.0 {
        target_state.is_alive = false;
    }
}

fn update_player_state(
    elapsed_time: Duration,
    player_prototype: &PlayerPrototype,
    player_state: &mut PlayerState,
) {
    if !player_state.character_state.is_alive {
        return;
    }

    update_character_state(
        elapsed_time,
        &player_prototype.character_prototype,
        &mut player_state.character_state,
    );

    player_state.mana = player_prototype
        .max_mana
        .min(player_state.mana + (elapsed_time.as_secs_f64() * player_prototype.mana_regen));
}

fn update_monster_states(
    elapsed_time: Duration,
    monster_prototypes: &Vec<MonsterPrototype>,
    monster_states: &mut Vec<MonsterState>,
) {
    for (monster_state, monster_prototype) in monster_states
        .iter_mut()
        .zip(monster_prototypes.iter())
        .filter(|(s, _)| s.character_state.is_alive)
    {
        monster_state.initiative = (monster_state.initiative - elapsed_time.as_secs_f32()).max(0.0);
        if monster_state.initiative > 0.0 {
            continue;
        }

        update_character_state(
            elapsed_time,
            &monster_prototype.character_prototype,
            &mut monster_state.character_state,
        );
    }
}

fn update_character_state(
    elapsed_time: Duration,
    prototype: &CharacterPrototype,
    state: &mut CharacterState,
) {
    if !state.is_alive {
        return;
    }

    state.health = prototype
        .max_health
        .min(state.health + (elapsed_time.as_secs_f64() * prototype.health_regen));

    for (skill_prototype, skill_state) in prototype
        .skill_prototypes
        .iter()
        .zip(state.skill_states.iter_mut())
    {
        skill_state.elapsed_cooldown += elapsed_time.as_secs_f32();
        if skill_state.elapsed_cooldown >= skill_prototype.cooldown {
            skill_state.elapsed_cooldown = skill_prototype.cooldown;
            skill_state.is_ready = true;
        } else {
            skill_state.is_ready = false;
        }
    }
}
