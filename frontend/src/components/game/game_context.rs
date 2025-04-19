use leptos::prelude::*;

use shared::data::MonsterPrototype;
use shared::data::MonsterState;
use shared::data::PlayerPrototype;
use shared::data::PlayerState;

#[derive(Clone)]
pub struct GameContext {
    pub started: RwSignal<bool>,

    pub player_prototype: RwSignal<PlayerPrototype>,
    pub player_state: RwSignal<PlayerState>,

    pub monster_wave: RwSignal<usize>, // Used to generate unique key in list
    pub monster_prototypes: RwSignal<Vec<MonsterPrototype>>,
    pub monster_states: RwSignal<Vec<MonsterState>>,
}

impl GameContext {
    pub fn new() -> Self {
        GameContext {
            started: RwSignal::new(false),
            player_prototype: RwSignal::new(PlayerPrototype::default()),
            player_state: RwSignal::new(PlayerState::default()),
            monster_wave: RwSignal::new(0),
            monster_prototypes: RwSignal::new(Vec::new()),
            monster_states: RwSignal::new(Vec::new()),
        }
    }
}
