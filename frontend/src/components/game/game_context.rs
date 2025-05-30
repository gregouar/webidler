use leptos::prelude::*;

use shared::data::{
    game_stats::GameStats,
    item::ItemCategory,
    loot::QueuedLoot,
    monster::{MonsterSpecs, MonsterState},
    passive::{PassivesTreeSpecs, PassivesTreeState},
    player::{PlayerInventory, PlayerResources, PlayerSpecs, PlayerState},
    world::{WorldSpecs, WorldState},
};

#[derive(Clone)]
pub struct GameContext {
    pub started: RwSignal<bool>,

    pub world_specs: RwSignal<WorldSpecs>,
    pub world_state: RwSignal<WorldState>,

    pub passives_tree_specs: RwSignal<PassivesTreeSpecs>,
    pub passives_tree_state: RwSignal<PassivesTreeState>,

    pub player_specs: RwSignal<PlayerSpecs>,
    pub player_inventory: RwSignal<PlayerInventory>,
    pub player_state: RwSignal<PlayerState>,
    pub player_resources: RwSignal<PlayerResources>,

    pub monster_wave: RwSignal<usize>, // Used to generate unique key in list
    pub monster_specs: RwSignal<Vec<MonsterSpecs>>,
    pub monster_states: RwSignal<Vec<MonsterState>>,

    pub queued_loot: RwSignal<Vec<QueuedLoot>>,

    pub game_stats: RwSignal<GameStats>,

    // TODO: Is this really the correct place? Should we have a UI context?
    // TODO: enum ?
    pub open_inventory: RwSignal<bool>,
    pub open_passives: RwSignal<bool>,
    pub open_statistics: RwSignal<bool>,

    pub loot_preference: RwSignal<Option<ItemCategory>>,
}

impl Default for GameContext {
    fn default() -> Self {
        Self::new()
    }
}

impl GameContext {
    pub fn new() -> Self {
        GameContext {
            started: RwSignal::new(false),

            world_specs: RwSignal::new(WorldSpecs::default()),
            world_state: RwSignal::new(WorldState::default()),

            passives_tree_specs: RwSignal::new(PassivesTreeSpecs::default()),
            passives_tree_state: RwSignal::new(PassivesTreeState::default()),

            player_specs: RwSignal::new(PlayerSpecs::default()),
            player_inventory: RwSignal::new(PlayerInventory::default()),
            player_state: RwSignal::new(PlayerState::default()),
            player_resources: RwSignal::new(PlayerResources::default()),

            monster_wave: RwSignal::new(0),
            monster_specs: RwSignal::new(Vec::new()),
            monster_states: RwSignal::new(Vec::new()),

            queued_loot: RwSignal::new(Vec::new()),

            game_stats: RwSignal::new(GameStats::default()),

            open_inventory: RwSignal::new(false),
            open_passives: RwSignal::new(false),
            open_statistics: RwSignal::new(false),
            loot_preference: RwSignal::new(None),
        }
    }
}
