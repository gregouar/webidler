use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use shared::data::{
    area::{AreaLevel, AreaSpecs, AreaState, AreaThreat},
    game_stats::GameStats,
    item::ItemSpecs,
    loot::QueuedLoot,
    monster::{MonsterSpecs, MonsterState},
    passive::{PassivesTreeSpecs, PassivesTreeState},
    player::{PlayerInventory, PlayerResources, PlayerSpecs, PlayerState},
    quest::QuestRewards,
    temple::PlayerBenedictions,
};

use crate::game::{
    data::{area::AreaBlueprint, master_store, DataInit},
    systems::{
        area_controller, benedictions_controller, passives_controller,
        player_controller::PlayerController, player_updater,
    },
    utils::LazySyncer,
};

#[derive(Debug, Clone)]
pub struct GameInstanceData {
    pub area_id: String,
    pub map_item: Option<ItemSpecs>,

    pub area_blueprint: AreaBlueprint,
    pub area_specs: AreaSpecs,
    pub area_state: LazySyncer<AreaState>,
    pub area_threat: AreaThreat,

    pub passives_tree_id: String,
    pub passives_tree_specs: PassivesTreeSpecs,
    pub passives_tree_state: LazySyncer<PassivesTreeState>,

    pub player_benedictions: PlayerBenedictions,

    pub player_specs: LazySyncer<PlayerSpecs>,
    pub player_inventory: LazySyncer<PlayerInventory>,
    pub player_state: PlayerState,
    pub player_resources: LazySyncer<PlayerResources>,
    pub player_stamina: Duration,

    pub player_controller: PlayerController,
    pub player_respawn_delay: Duration,

    // TODO: Need to find better more granular way to handle these things...
    // Probably need to split more between static and computed specs
    pub monster_base_specs: LazySyncer<Vec<MonsterSpecs>>,
    pub monster_specs: Vec<MonsterSpecs>, // Only use internally, not shared
    pub monster_states: Vec<MonsterState>,
    pub monster_wave_delay: Duration,

    pub wave_completed: bool,
    pub queued_loot: LazySyncer<Vec<QueuedLoot>>,

    pub game_stats: GameStats,

    pub end_quest: bool, // Initiate end, generate rewards
    pub quest_rewards: LazySyncer<Option<QuestRewards>>,
    pub terminate_quest: bool, // Actually close the quest
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedGameData {
    area_id: String,
    map_item: Option<ItemSpecs>,
    area_level: AreaLevel,
    max_area_level_completed: AreaLevel,
    passives_tree_id: String,
    passives_tree_state: PassivesTreeState,
    player_resources: PlayerResources,
    player_specs: PlayerSpecs,
    player_inventory: PlayerInventory,
    queued_loot: Vec<QueuedLoot>,
    game_stats: GameStats,
    last_champion_spawn: AreaLevel,
    auto_progress: bool,
    #[serde(default)] // For compatibility
    max_area_level: AreaLevel,
    #[serde(default)] // For compatibility
    player_benedictions: PlayerBenedictions,
    #[serde(default)]
    player_stamina: Duration,

    #[serde(default)]
    end_quest: bool,
    #[serde(default)]
    quest_rewards: Option<QuestRewards>,
}

impl std::ops::Deref for SavedGameData {
    type Target = GameStats;

    fn deref(&self) -> &Self::Target {
        &self.game_stats
    }
}

impl GameInstanceData {
    #[allow(clippy::too_many_arguments)]
    pub fn init_from_store(
        master_store: &master_store::MasterStore,
        area_id: &str,
        map_item: Option<ItemSpecs>,
        max_area_level_completed: AreaLevel,
        passives_tree_id: &str,
        mut passives_tree_state: PassivesTreeState,
        player_benedictions: PlayerBenedictions,
        mut player_resources: PlayerResources,
        mut player_specs: PlayerSpecs,
        player_inventory: PlayerInventory,
        player_stamina: Duration,
    ) -> Result<Self> {
        let mut area_blueprint = master_store
            .area_blueprints_store
            .get(area_id)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("couldn't load area: {}", area_id))?;

        let area_specs = area_controller::init_area_specs(
            &master_store.loot_tables_store,
            &mut area_blueprint,
            &map_item,
        );

        let mut area_state = AreaState::init(&area_specs);
        area_state.max_area_level_ever = max_area_level_completed;

        let mut passives_tree_specs = master_store
            .passives_store
            .get(passives_tree_id)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("couldn't load passives tree: {}", passives_tree_id))?;
        passives_controller::compute_passives_tree_specs(
            &mut passives_tree_specs,
            &passives_tree_state.ascension,
        );

        let player_state = PlayerState::init(&player_specs);
        passives_controller::refund_missing(
            &passives_tree_specs,
            &mut passives_tree_state,
            &mut player_resources,
        );

        player_updater::update_player_specs(
            &mut player_specs,
            &player_state,
            &player_inventory,
            &passives_tree_specs,
            &passives_tree_state,
            &benedictions_controller::generate_effects_map_from_benedictions(
                &master_store.benedictions_store,
                &player_benedictions,
            ),
            &AreaThreat::default(),
        );

        Ok(Self {
            area_id: area_id.to_string(),
            map_item,
            area_specs,
            area_state: LazySyncer::new(area_state),
            area_blueprint,
            area_threat: AreaThreat::default(),

            passives_tree_id: passives_tree_id.to_string(),
            passives_tree_specs,
            passives_tree_state: LazySyncer::new(passives_tree_state),

            player_benedictions,

            player_resources: LazySyncer::new(player_resources),
            player_state: PlayerState::init(&player_specs),
            player_controller: PlayerController::init(&player_specs),
            player_specs: LazySyncer::new(player_specs),
            player_inventory: LazySyncer::new(player_inventory),
            player_respawn_delay: Default::default(),
            player_stamina,

            monster_base_specs: LazySyncer::new(Vec::new()),
            monster_specs: Vec::new(),
            monster_states: Vec::new(),
            monster_wave_delay: Default::default(),

            wave_completed: false,
            queued_loot: LazySyncer::new(Default::default()),

            game_stats: Default::default(),

            end_quest: false,
            quest_rewards: LazySyncer::new(None),
            terminate_quest: false,
        })
    }

    pub fn to_bytes(self) -> Result<Vec<u8>> {
        Ok(rmp_serde::to_vec(&SavedGameData {
            area_id: self.area_id,
            map_item: self.map_item,
            area_level: self.area_state.read().area_level,
            max_area_level: self.area_state.read().max_area_level,
            max_area_level_completed: self.area_state.read().max_area_level_ever,
            passives_tree_id: self.passives_tree_id,
            passives_tree_state: self.passives_tree_state.unwrap(),
            player_benedictions: self.player_benedictions,
            player_resources: self.player_resources.unwrap(),
            player_specs: self.player_specs.unwrap(),
            player_inventory: self.player_inventory.unwrap(),
            player_stamina: self.player_stamina,
            queued_loot: self.queued_loot.unwrap(),
            game_stats: self.game_stats,
            last_champion_spawn: self.area_state.read().last_champion_spawn,
            auto_progress: self.area_state.read().auto_progress,
            end_quest: self.end_quest,
            quest_rewards: self.quest_rewards.read().clone(),
        })?)
    }

    pub fn from_bytes(master_store: &master_store::MasterStore, bytes: &[u8]) -> Result<Self> {
        let SavedGameData {
            area_id,
            map_item,
            area_level,
            max_area_level,
            max_area_level_completed,
            passives_tree_id,
            passives_tree_state,
            player_benedictions,
            player_resources,
            player_specs,
            player_inventory,
            player_stamina,
            queued_loot,
            game_stats,
            last_champion_spawn,
            auto_progress,
            end_quest,
            quest_rewards,
        } = rmp_serde::from_slice::<SavedGameData>(bytes)?;

        let mut s = Self::init_from_store(
            master_store,
            &area_id,
            map_item,
            max_area_level_completed,
            &passives_tree_id,
            passives_tree_state,
            player_benedictions,
            player_resources,
            player_specs,
            player_inventory,
            player_stamina,
        )?;

        s.area_state.mutate().area_level = area_level;
        s.area_state.mutate().max_area_level = max_area_level;
        s.area_state.mutate().last_champion_spawn = last_champion_spawn;
        s.area_state.mutate().auto_progress = auto_progress;
        s.queued_loot.mutate().extend(queued_loot);
        s.game_stats = game_stats;
        s.end_quest = end_quest;
        *s.quest_rewards.mutate() = quest_rewards;

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
        self.quest_rewards.mutate();
    }
}
