use futures::lock::Mutex;
use indexmap::IndexMap;
use leptos::prelude::*;
use std::{collections::HashMap, sync::Arc};

use shared::data::{
    achievements::AchievementSpecs,
    area::AreaSpecs,
    character_status::{StatusId, StatusSpecs},
    cosmetics::CosmeticType,
    item::{BaseItemId, ItemBase, ItemSpecs},
    item_affix::ItemAffixBlueprint,
    passive::PassivesTreeSpecs,
    pets::PetSpecs,
    skill::BaseSkillSpecs,
    skill_mastery::SkillMasterySpecs,
    temple::BenedictionsCategory,
};

use crate::components::backend_client::{BackendClient, BackendError};

#[derive(Clone, Copy)]
pub struct DataContext {
    pub areas_specs: RwSignal<HashMap<String, AreaSpecs>>,
    pub skill_specs: RwSignal<HashMap<String, BaseSkillSpecs>>,
    pub skill_mastery_specs: RwSignal<IndexMap<String, SkillMasterySpecs>>,
    pub statuses_specs: RwSignal<HashMap<StatusId, StatusSpecs>>,
    pub cosmetics_specs: RwSignal<HashMap<String, CosmeticType>>,
    pub pets_specs: RwSignal<HashMap<String, PetSpecs>>,
    pub achievements: RwSignal<IndexMap<String, AchievementSpecs>>,
    pub passives_tree_specs: RwSignal<PassivesTreeSpecs>,
    pub benedictions_specs: RwSignal<IndexMap<String, BenedictionsCategory>>,
    pub item_bases: RwSignal<HashMap<BaseItemId, Arc<ItemBase>>>,
    pub item_affixes: RwSignal<HashMap<String, ItemAffixBlueprint>>,

    default_item_base: StoredValue<Arc<ItemBase>>,

    load_lock: StoredValue<Arc<Mutex<()>>>,
    loaded: RwSignal<bool>,
}

pub fn provide_data_context() {
    provide_context(DataContext {
        areas_specs: RwSignal::new(Default::default()),
        skill_specs: RwSignal::new(Default::default()),
        skill_mastery_specs: RwSignal::new(Default::default()),
        statuses_specs: RwSignal::new(Default::default()),
        cosmetics_specs: RwSignal::new(Default::default()),
        pets_specs: RwSignal::new(Default::default()),
        achievements: RwSignal::new(Default::default()),
        passives_tree_specs: RwSignal::new(Default::default()),
        benedictions_specs: RwSignal::new(Default::default()),
        item_bases: RwSignal::new(Default::default()),
        item_affixes: RwSignal::new(Default::default()),
        load_lock: StoredValue::new(Arc::new(Mutex::new(()))),
        loaded: RwSignal::new(false),
        default_item_base: StoredValue::new(Arc::new(ItemBase {
            name: "Item".to_string(),
            icon: "ui/shield.webp".to_string(),
            description: Default::default(),
            slot: Default::default(),
            extra_slots: Default::default(),
            categories: Default::default(),
            min_area_level: Default::default(),
            rarity: Default::default(),
            affixes: Default::default(),
            triggers: Default::default(),
            weapon_specs: Default::default(),
            armor_specs: Default::default(),
            rune_specs: Default::default(),
            map_specs: Default::default(),
            ignore_quality: Default::default(),
            upgrade_levels: Default::default(),
            upgrade_effects: Default::default(),
        })),
    });
}

impl DataContext {
    pub async fn load_data(&self, backend_client: BackendClient) -> Result<(), BackendError> {
        // Concurrent consumers share the first successful load.
        let load_lock = self.load_lock.get_value();
        let _guard = load_lock.lock().await;

        if self.loaded.get_untracked() {
            return Ok(());
        }

        let data = backend_client.get_master_data().await?;

        self.areas_specs.set(data.areas);
        self.skill_specs.set(data.skills);
        self.skill_mastery_specs.set(data.skill_masteries);
        self.statuses_specs.set(data.statuses);
        self.cosmetics_specs.set(data.cosmetics);
        self.pets_specs.set(data.pets);
        self.achievements.set(data.achievements);
        self.passives_tree_specs.set(data.passives_tree_specs);
        self.benedictions_specs.set(data.benedictions_specs);
        self.item_bases.set(
            data.item_bases
                .into_iter()
                .map(|(id, item)| (id, Arc::new(item)))
                .collect(),
        );
        self.item_affixes.set(data.item_affixes);

        self.loaded.set(true);

        Ok(())
    }

    pub fn skill_name(&self, skill_id: &str) -> String {
        self.skill_specs
            .read_untracked()
            .get(skill_id)
            .map(|skill| skill.name.clone())
            .unwrap_or(skill_id.to_string())
    }

    pub fn status_name(&self, status_id: &StatusId) -> String {
        self.statuses_specs
            .read_untracked()
            .get(status_id)
            .map(|status_specs| status_specs.name.clone())
            .unwrap_or(status_id.to_string())
    }

    pub fn status_adjective(&self, status_id: &StatusId) -> Option<String> {
        self.statuses_specs
            .read_untracked()
            .get(status_id)
            .and_then(|status_specs| status_specs.adjective.clone())
    }

    pub fn get_item_base(&self, item_specs: &ItemSpecs) -> Arc<ItemBase> {
        self.item_bases
            .read_untracked()
            .get(&item_specs.modifiers.base_item_id)
            .cloned()
            .unwrap_or(self.default_item_base.get_value())
    }
}
