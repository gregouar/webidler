use std::time::Duration;

use indexmap::IndexSet;
use leptos::prelude::{
    guards::{Plain, ReadGuard},
    *,
};

use shared::data::{
    area::{AreaSpecs, AreaState, AreaThreat},
    game_stats::GameStats,
    item::ItemCategory,
    loot::QueuedLoot,
    monster::{MonsterSpecs, MonsterState},
    passive::{PassivesTreeSpecs, PassivesTreeState, PurchasedNodes},
    player::{PlayerInventory, PlayerResources, PlayerSpecs, PlayerState},
    quest::QuestRewards,
};

use crate::{components::game::local_stats::GameLocalStats, utils};

#[derive(Clone, Copy)]
pub struct GameContext {
    pub started: RwSignal<bool>,

    pub area_specs: RwSignal<AreaSpecs>,
    pub area_state: Syncable<AreaState>,
    pub area_threat: RwSignal<AreaThreat>,

    pub passives_tree_specs: RwSignal<PassivesTreeSpecs>,
    pub passives_tree_state: Syncable<PassivesTreeState>,
    pub passives_tree_build: Syncable<PurchasedNodes>,

    pub player_specs: Syncable<PlayerSpecs>,
    pub player_inventory: RwSignal<PlayerInventory>,
    pub player_state: RwSignal<PlayerState>,
    pub player_resources: Syncable<PlayerResources>,
    pub player_stamina: RwSignal<Duration>,

    pub monster_wave: RwSignal<usize>, // Used to generate unique key in list
    pub monster_specs: RwSignal<Vec<MonsterSpecs>>,
    pub monster_states: RwSignal<Vec<MonsterState>>,

    pub queued_loot: Syncable<Vec<QueuedLoot>>,
    pub quest_rewards: RwSignal<Option<QuestRewards>>,

    pub game_stats: RwSignal<GameStats>,
    pub game_local_stats: GameLocalStats,

    // TODO: Is this really the correct place? Should we have a UI context?
    // TODO: enum ?
    pub open_inventory: RwSignal<bool>,
    pub open_passives: RwSignal<bool>,
    pub open_statistics: RwSignal<bool>,
    pub open_skills: RwSignal<bool>,
    pub open_end_quest: RwSignal<bool>,

    pub loot_preference: RwSignal<Option<ItemCategory>>,
    pub last_skills_bought: RwSignal<IndexSet<String>>,
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

            area_specs: RwSignal::new(Default::default()),
            area_state: Default::default(),
            area_threat: RwSignal::new(Default::default()),

            passives_tree_specs: RwSignal::new(Default::default()),
            passives_tree_state: Default::default(),
            passives_tree_build: Default::default(),

            player_specs: Default::default(),
            player_inventory: RwSignal::new(Default::default()),
            player_state: RwSignal::new(Default::default()),
            player_resources: Default::default(),
            player_stamina: Default::default(),

            monster_wave: RwSignal::new(0),
            monster_specs: RwSignal::new(Vec::new()),
            monster_states: RwSignal::new(Vec::new()),

            queued_loot: Default::default(),
            quest_rewards: RwSignal::new(None),

            game_stats: RwSignal::new(Default::default()),
            game_local_stats: Default::default(),

            open_inventory: RwSignal::new(false),
            open_passives: RwSignal::new(false),
            open_statistics: RwSignal::new(false),
            open_skills: RwSignal::new(false),
            open_end_quest: RwSignal::new(false),

            loot_preference: RwSignal::new(None),
            last_skills_bought: RwSignal::new(Default::default()),
        }
    }
}

#[derive(Clone)]
pub struct Syncable<T> {
    client_value: RwSignal<T>,
    server_value: RwSignal<Option<T>>,

    server_update_time: RwSignal<Option<f64>>,
    client_update_time: RwSignal<Option<f64>>,
}

impl<T> Copy for Syncable<T> where T: Default + Clone + Send + Sync + 'static {}

impl<T> Syncable<T>
where
    T: Default + Clone + Send + Sync + 'static,
{
    pub fn new() -> Self {
        Self {
            client_value: RwSignal::new(Default::default()),
            server_value: RwSignal::new(None),
            server_update_time: RwSignal::new(None),
            client_update_time: RwSignal::new(None),
        }
    }

    // Leptos signal wrapper for client-side updates
    pub fn get(&self) -> T {
        self.client_value.get()
    }

    pub fn read(&self) -> ReadGuard<T, Plain<T>> {
        self.client_value.read()
    }

    pub fn read_untracked(&self) -> ReadGuard<T, Plain<T>> {
        self.client_value.read_untracked()
    }

    pub fn with<U>(&self, fun: impl FnOnce(&T) -> U) -> U {
        self.client_value.with(fun)
    }

    pub fn set(&self, value: T) {
        self.client_update_time.set(Some(utils::now()));
        self.client_value.set(value);
    }

    pub fn write(&self) -> impl UntrackableGuard<Target = T> {
        self.client_update_time.set(Some(utils::now()));
        self.client_value.write()
    }

    pub fn update(&self, fun: impl FnOnce(&mut T)) {
        self.client_update_time.set(Some(utils::now()));
        self.client_value.update(fun);
    }

    // Sync client-side with server-side, adding delay if recent client update
    pub fn sync(&self, server_value: Option<T>) {
        if let Some(server_value) = server_value {
            self.server_update_time.set(Some(utils::now()));
            self.server_value.set(Some(server_value));
        }
        self.debounce();
    }

    fn debounce(&self) {
        let server_time = self.server_update_time.get_untracked();
        if let Some((server_time, client_time)) =
            server_time.zip(self.client_update_time.get_untracked())
        {
            if server_time > client_time && (utils::now() - client_time) > 500.0 {
                self.sync_value();
            }
        } else if server_time.is_some() {
            self.sync_value();
        }
    }

    fn sync_value(&self) {
        if let Some(server_value) = self.server_value.write().take() {
            self.client_value.set(server_value);
        }
    }
}

impl<T> Default for Syncable<T>
where
    T: Default + Clone + Send + Sync + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}
