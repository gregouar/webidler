use leptos::prelude::*;

use shared::data::{MonsterSpecs, MonsterState, PlayerSpecs, PlayerState, WorldSpecs, WorldState};

#[derive(Clone)]
pub struct GameContext {
    pub started: RwSignal<bool>,

    pub world_specs: RwSignal<WorldSpecs>,
    pub world_state: RwSignal<WorldState>,

    pub player_specs: RwSignal<PlayerSpecs>,
    pub player_state: RwSignal<PlayerState>,

    pub monster_wave: RwSignal<usize>, // Used to generate unique key in list
    pub monster_specs: RwSignal<Vec<MonsterSpecs>>,
    pub monster_states: RwSignal<Vec<MonsterState>>,

    // TODO: Is this really the correct place? Should we have a UI context?
    pub open_inventory: RwSignal<bool>,
}

impl GameContext {
    pub fn new() -> Self {
        GameContext {
            started: RwSignal::new(false),
            world_specs: RwSignal::new(WorldSpecs::default()),
            world_state: RwSignal::new(WorldState::default()),
            player_specs: RwSignal::new(PlayerSpecs::default()),
            player_state: RwSignal::new(PlayerState::default()),
            monster_wave: RwSignal::new(0),
            monster_specs: RwSignal::new(Vec::new()),
            monster_states: RwSignal::new(Vec::new()),
            open_inventory: RwSignal::new(false),
        }
    }
}
