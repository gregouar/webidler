use leptos::prelude::*;
use std::collections::HashMap;

use shared::data::{area::AreaSpecs, skill::BaseSkillSpecs};

use crate::components::backend_client::{BackendClient, BackendError};

#[derive(Clone, Copy)]
pub struct DataContext {
    pub areas_specs: RwSignal<HashMap<String, AreaSpecs>>,
    pub skill_specs: RwSignal<HashMap<String, BaseSkillSpecs>>,
    pub loaded: RwSignal<bool>,
}

pub fn provide_data_context() {
    provide_context(DataContext {
        areas_specs: RwSignal::new(Default::default()),
        skill_specs: RwSignal::new(Default::default()),
        loaded: RwSignal::new(false),
    });
}

impl DataContext {
    pub async fn load_data(&self, backend_client: BackendClient) -> Result<(), BackendError> {
        if self.loaded.get_untracked() {
            return Ok(());
        }

        let (areas, skills) =
            futures::join!(backend_client.get_areas(), backend_client.get_skills());

        self.areas_specs.set(areas?.areas);
        self.skill_specs.set(skills?.skills);

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
}
