use std::time::Instant;

use super::data::DataInit;
use super::systems::player_controller::PlayerController;
use super::{data::world::WorldBlueprint, utils::LazySyncer};

use shared::data::game_stats::GameStats;
use shared::data::passive::{PassivesTreeSpecs, PassivesTreeState};
use shared::data::{
    loot::QueuedLoot,
    monster::{MonsterSpecs, MonsterState},
    player::{PlayerInventory, PlayerResources, PlayerSpecs, PlayerState},
    world::WorldState,
};

#[derive(Debug, Clone)]
pub struct GameInstanceData {
    pub world_blueprint: WorldBlueprint,
    pub world_state: LazySyncer<WorldState>,

    pub passives_tree_specs: PassivesTreeSpecs,
    pub passives_tree_state: LazySyncer<PassivesTreeState>,

    pub player_specs: LazySyncer<PlayerSpecs>,
    pub player_inventory: LazySyncer<PlayerInventory>,
    pub player_state: PlayerState,
    pub player_resources: LazySyncer<PlayerResources>,

    pub player_controller: PlayerController,
    pub player_respawn_delay: Instant,

    // TODO: Need to find better more granular way to handle these things...
    // Probably need to split more between static and computed specs
    pub monster_base_specs: Vec<MonsterSpecs>,
    pub monster_specs: LazySyncer<Vec<MonsterSpecs>>,
    pub monster_states: Vec<MonsterState>,
    pub monster_wave_delay: Instant,

    pub wave_completed: bool,
    pub queued_loot: LazySyncer<Vec<QueuedLoot>>,

    pub game_stats: GameStats,
}

impl GameInstanceData {
    pub fn init(
        world_blueprint: WorldBlueprint,
        passives_tree_specs: PassivesTreeSpecs,
        player_resources: PlayerResources,
        player_specs: PlayerSpecs,
        player_inventory: PlayerInventory,
    ) -> Self {
        Self {
            world_state: LazySyncer::new(WorldState::init(&world_blueprint.specs)),
            world_blueprint,

            passives_tree_state: LazySyncer::new(PassivesTreeState::default()),
            passives_tree_specs,

            player_resources: LazySyncer::new(player_resources),
            player_state: PlayerState::init(&player_specs),
            player_controller: PlayerController::init(&player_specs),
            player_specs: LazySyncer::new(player_specs),
            player_inventory: LazySyncer::new(player_inventory),
            player_respawn_delay: Instant::now(),

            monster_base_specs: Vec::new(),
            monster_specs: LazySyncer::new(Vec::new()),
            monster_states: Vec::new(),
            monster_wave_delay: Instant::now(),

            wave_completed: false,
            queued_loot: LazySyncer::new(Vec::new()),

            game_stats: GameStats::default(),
        }
    }

    pub fn reset_syncers(&mut self) {
        self.world_state.mutate();
        self.passives_tree_state.mutate();
        self.player_resources.mutate();
        self.player_specs.mutate();
        self.player_inventory.mutate();
        self.monster_specs.mutate();
        self.queued_loot.mutate();
    }
}
