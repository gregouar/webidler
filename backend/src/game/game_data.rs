use anyhow::Result;
use std::time::Instant;

use crate::game::data::master_store;

use super::data::DataInit;
use super::systems::player_controller::PlayerController;
use super::{data::world::WorldBlueprint, utils::LazySyncer};

use serde::{Deserialize, Serialize};
use shared::data::game_stats::GameStats;
use shared::data::passive::{PassivesTreeSpecs, PassivesTreeState};
use shared::data::world::AreaLevel;
use shared::data::{
    loot::QueuedLoot,
    monster::{MonsterSpecs, MonsterState},
    player::{PlayerInventory, PlayerResources, PlayerSpecs, PlayerState},
    world::WorldState,
};

#[derive(Debug, Clone)]
pub struct GameInstanceData {
    pub world_id: String,
    pub world_blueprint: WorldBlueprint,
    pub world_state: LazySyncer<WorldState>,

    pub passives_tree_id: String,
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
    pub monster_base_specs: LazySyncer<Vec<MonsterSpecs>>,
    pub monster_specs: Vec<MonsterSpecs>, // Only use internally, not shared
    pub monster_states: Vec<MonsterState>,
    pub monster_wave_delay: Instant,

    pub wave_completed: bool,
    pub queued_loot: LazySyncer<Vec<QueuedLoot>>,

    pub game_stats: GameStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedGameData {
    pub world_id: String,
    pub area_level: AreaLevel,
    pub passives_tree_id: String,
    pub passives_tree_state: PassivesTreeState,
    pub player_resources: PlayerResources,
    pub player_specs: PlayerSpecs,
    pub player_inventory: PlayerInventory,
    pub queued_loot: Vec<QueuedLoot>,
    pub game_stats: GameStats,
}

impl GameInstanceData {
    pub fn new(
        world_id: String,
        world_blueprint: WorldBlueprint,
        area_level: Option<AreaLevel>,
        passives_tree_id: String,
        passives_tree_specs: PassivesTreeSpecs,
        passives_tree_state: Option<PassivesTreeState>,
        player_resources: PlayerResources,
        player_specs: PlayerSpecs,
        player_inventory: PlayerInventory,
        queued_loot: Option<Vec<QueuedLoot>>,
        game_stats: Option<GameStats>,
    ) -> Self {
        let mut world_state = WorldState::init(&world_blueprint.specs);
        if let Some(area_level) = area_level {
            world_state.area_level = area_level;
        }
        Self {
            world_id,
            world_state: LazySyncer::new(world_state),
            world_blueprint,

            passives_tree_id,
            passives_tree_specs,
            passives_tree_state: LazySyncer::new(passives_tree_state.unwrap_or_default()),

            player_resources: LazySyncer::new(player_resources),
            player_state: PlayerState::init(&player_specs),
            player_controller: PlayerController::init(&player_specs),
            player_specs: LazySyncer::new(player_specs),
            player_inventory: LazySyncer::new(player_inventory),
            player_respawn_delay: Instant::now(),

            monster_base_specs: LazySyncer::new(Vec::new()),
            monster_specs: Vec::new(),
            monster_states: Vec::new(),
            monster_wave_delay: Instant::now(),

            wave_completed: false,
            queued_loot: LazySyncer::new(queued_loot.unwrap_or_default()),

            game_stats: game_stats.unwrap_or_default(),
        }
    }

    pub fn init_from_store(
        master_store: &master_store::MasterStore,
        world_id: &str,
        area_level: Option<AreaLevel>,
        passives_tree_id: &str,
        passives_tree_state: Option<PassivesTreeState>,
        player_resources: PlayerResources,
        player_specs: PlayerSpecs,
        player_inventory: PlayerInventory,
        queued_loot: Option<Vec<QueuedLoot>>,
        game_stats: Option<GameStats>,
    ) -> Result<Self> {
        let world_blueprint = master_store
            .world_blueprints_store
            .get(world_id)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("couldn't load world: {}", world_id))?;

        let passives_tree_specs = master_store
            .passives_store
            .get(passives_tree_id)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("couldn't load passives tree: {}", passives_tree_id))?;

        Ok(Self::new(
            world_id.to_string(),
            world_blueprint,
            area_level,
            passives_tree_id.to_string(),
            passives_tree_specs,
            passives_tree_state,
            player_resources,
            player_specs,
            player_inventory,
            queued_loot,
            game_stats,
        ))
    }

    pub fn to_bytes(self) -> Result<Vec<u8>> {
        Ok(rmp_serde::to_vec(&SavedGameData {
            world_id: self.world_id,
            area_level: self.world_state.read().area_level,
            passives_tree_id: self.passives_tree_id,
            passives_tree_state: self.passives_tree_state.unwrap(),
            player_resources: self.player_resources.unwrap(),
            player_specs: self.player_specs.unwrap(),
            player_inventory: self.player_inventory.unwrap(),
            queued_loot: self.queued_loot.unwrap(),
            game_stats: self.game_stats,
        })?)
    }

    pub fn from_bytes(master_store: &master_store::MasterStore, bytes: &[u8]) -> Result<Self> {
        let SavedGameData {
            world_id,
            area_level,
            passives_tree_id,
            passives_tree_state,
            player_resources,
            player_specs,
            player_inventory,
            queued_loot,
            game_stats,
        } = rmp_serde::from_slice::<SavedGameData>(bytes)?;

        Self::init_from_store(
            master_store,
            &world_id,
            Some(area_level),
            &passives_tree_id,
            Some(passives_tree_state),
            player_resources,
            player_specs,
            player_inventory,
            Some(queued_loot),
            Some(game_stats),
        )
    }

    pub fn reset_syncers(&mut self) {
        self.world_state.mutate();
        self.passives_tree_state.mutate();
        self.player_resources.mutate();
        self.player_specs.mutate();
        self.player_inventory.mutate();
        self.monster_base_specs.mutate();
        self.queued_loot.mutate();
    }
}
