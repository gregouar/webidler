use anyhow::Result;
use std::time::Instant;

use crate::game::data::master_store;
use crate::game::systems::player_updater;

use super::data::DataInit;
use super::systems::player_controller::PlayerController;
use super::{data::area::AreaBlueprint, utils::LazySyncer};

use serde::{Deserialize, Serialize};
use shared::data::area::{AreaLevel, AreaThreat};
use shared::data::game_stats::GameStats;
use shared::data::passive::{PassivesTreeSpecs, PassivesTreeState};
use shared::data::{
    area::AreaState,
    loot::QueuedLoot,
    monster::{MonsterSpecs, MonsterState},
    player::{PlayerInventory, PlayerResources, PlayerSpecs, PlayerState},
};

#[derive(Debug, Clone)]
pub struct GameInstanceData {
    pub area_id: String,
    pub area_blueprint: AreaBlueprint,
    pub area_state: LazySyncer<AreaState>,
    pub area_threat: AreaThreat,

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
    pub area_id: String,
    pub area_level: AreaLevel,
    pub max_area_level_completed: AreaLevel,
    pub passives_tree_id: String,
    pub passives_tree_state: PassivesTreeState,
    pub player_resources: PlayerResources,
    pub player_specs: PlayerSpecs,
    pub player_inventory: PlayerInventory,
    pub queued_loot: Vec<QueuedLoot>,
    pub game_stats: GameStats,
    pub last_champion_spawn: AreaLevel,
    pub auto_progress: bool,
}

impl GameInstanceData {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        area_id: String,
        area_blueprint: AreaBlueprint,
        max_area_level_completed: AreaLevel,
        passives_tree_id: String,
        passives_tree_specs: PassivesTreeSpecs,
        passives_tree_state: PassivesTreeState,
        player_resources: PlayerResources,
        player_specs: PlayerSpecs,
        player_inventory: PlayerInventory,
    ) -> Self {
        let mut area_state = AreaState::init(&area_blueprint.specs);
        area_state.max_area_level_completed = max_area_level_completed;

        let mut game_data = Self {
            area_id,
            area_state: LazySyncer::new(area_state),
            area_blueprint,
            area_threat: AreaThreat::default(),

            passives_tree_id,
            passives_tree_specs,
            passives_tree_state: LazySyncer::new(passives_tree_state),

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
            queued_loot: LazySyncer::new(Default::default()),

            game_stats: Default::default(),
        };

        player_updater::update_player_specs(
            game_data.player_specs.mutate(),
            &game_data.player_state,
            game_data.player_inventory.read(),
            &game_data.passives_tree_specs,
            game_data.passives_tree_state.read(),
        );

        game_data.player_state = PlayerState::init(game_data.player_specs.read());

        game_data
    }

    #[allow(clippy::too_many_arguments)]
    pub fn init_from_store(
        master_store: &master_store::MasterStore,
        area_id: &str,
        max_area_level_completed: AreaLevel,
        passives_tree_id: &str,
        passives_tree_state: PassivesTreeState,
        player_resources: PlayerResources,
        player_specs: PlayerSpecs,
        player_inventory: PlayerInventory,
    ) -> Result<Self> {
        let area_blueprint = master_store
            .area_blueprints_store
            .get(area_id)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("couldn't load area: {}", area_id))?;

        let passives_tree_specs = master_store
            .passives_store
            .get(passives_tree_id)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("couldn't load passives tree: {}", passives_tree_id))?;

        Ok(Self::new(
            area_id.to_string(),
            area_blueprint,
            max_area_level_completed,
            passives_tree_id.to_string(),
            passives_tree_specs,
            passives_tree_state,
            player_resources,
            player_specs,
            player_inventory,
        ))
    }

    pub fn to_bytes(self) -> Result<Vec<u8>> {
        Ok(rmp_serde::to_vec(&SavedGameData {
            area_id: self.area_id,
            area_level: self.area_state.read().area_level,
            max_area_level_completed: self.area_state.read().max_area_level_completed,
            passives_tree_id: self.passives_tree_id,
            passives_tree_state: self.passives_tree_state.unwrap(),
            player_resources: self.player_resources.unwrap(),
            player_specs: self.player_specs.unwrap(),
            player_inventory: self.player_inventory.unwrap(),
            queued_loot: self.queued_loot.unwrap(),
            game_stats: self.game_stats,
            last_champion_spawn: self.area_state.read().last_champion_spawn,
            auto_progress: self.area_state.read().auto_progress,
        })?)
    }

    pub fn from_bytes(master_store: &master_store::MasterStore, bytes: &[u8]) -> Result<Self> {
        let SavedGameData {
            area_id,
            area_level,
            max_area_level_completed,
            passives_tree_id,
            passives_tree_state,
            player_resources,
            player_specs,
            player_inventory,
            queued_loot,
            game_stats,
            last_champion_spawn,
            auto_progress,
        } = rmp_serde::from_slice::<SavedGameData>(bytes)?;

        let mut s = Self::init_from_store(
            master_store,
            &area_id,
            max_area_level_completed,
            &passives_tree_id,
            passives_tree_state,
            player_resources,
            player_specs,
            player_inventory,
        )?;

        s.area_state.mutate().area_level = area_level;
        s.area_state.mutate().last_champion_spawn = last_champion_spawn;
        s.area_state.mutate().auto_progress = auto_progress;
        s.queued_loot.mutate().extend(queued_loot);
        s.game_stats = game_stats;

        Ok(s)
    }

    pub fn reset_syncers(&mut self) {
        self.area_state.mutate();
        self.passives_tree_state.mutate();
        self.player_resources.mutate();
        self.player_specs.mutate();
        self.player_inventory.mutate();
        self.monster_base_specs.mutate();
        self.queued_loot.mutate();
    }
}
