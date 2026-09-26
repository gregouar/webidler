use indexmap::IndexMap;
use leptos::prelude::*;
use std::collections::HashMap;

use shared::data::{
    achievements::AchievementSpecs,
    area::AreaSpecs,
    character_status::{StatusId, StatusSpecs},
    cosmetics::CosmeticType,
    pets::PetSpecs,
    skill::BaseSkillSpecs,
    skill_mastery::SkillMasterySpecs,
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
    pub loaded: RwSignal<bool>,
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
        loaded: RwSignal::new(false),
    });
}

impl DataContext {
    pub async fn load_data(&self, backend_client: BackendClient) -> Result<(), BackendError> {
        if self.loaded.get_untracked() {
            return Ok(());
        }

        let (areas, skills, statuses, cosmetics, pets, achievements) = futures::join!(
            backend_client.get_areas(),
            backend_client.get_skills(),
            backend_client.get_statuses(),
            backend_client.get_cosmetics(),
            backend_client.get_pets(),
            backend_client.get_achievements(),
        );

        self.areas_specs.set(areas?.areas);
        let skills = skills?;
        self.skill_specs.set(skills.skills);
        self.skill_mastery_specs.set(skills.skill_masteries);
        self.statuses_specs.set(statuses?.statuses);
        self.cosmetics_specs.set(cosmetics?.cosmetics);
        self.pets_specs.set(pets?.pets);
        self.achievements.set(achievements?.achievements);

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
}
