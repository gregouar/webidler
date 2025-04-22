use leptos::prelude::*;

use shared::data::MonsterSpecs;
use shared::data::MonsterState;
use shared::data::PlayerSpecs;
use shared::data::PlayerState;

#[derive(Clone)]
pub struct GameContext {
    pub started: RwSignal<bool>,

    pub player_specs: RwSignal<PlayerSpecs>,
    pub player_state: RwSignal<PlayerState>,

    pub monster_wave: RwSignal<usize>, // Used to generate unique key in list
    pub monster_specs: RwSignal<Vec<MonsterSpecs>>,
    pub monster_states: RwSignal<Vec<MonsterState>>,

    // Is this really the correct place?
    pub open_inventory: RwSignal<bool>,
}

impl GameContext {
    pub fn new() -> Self {
        GameContext {
            started: RwSignal::new(false),
            player_specs: RwSignal::new(PlayerSpecs::default()),
            player_state: RwSignal::new(PlayerState::default()),
            monster_wave: RwSignal::new(0),
            monster_specs: RwSignal::new(Vec::new()),
            monster_states: RwSignal::new(Vec::new()),
            open_inventory: RwSignal::new(false),
        }
    }
}
