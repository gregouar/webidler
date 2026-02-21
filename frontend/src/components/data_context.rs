use leptos::prelude::*;
use std::collections::HashMap;

use shared::data::area::AreaSpecs;

use crate::components::backend_client::{BackendClient, BackendError};

#[derive(Clone, Copy)]
pub struct DataContext {
    pub areas_specs: RwSignal<HashMap<String, AreaSpecs>>,
    pub loaded: RwSignal<bool>,
}

pub fn provide_data_context() {
    provide_context(DataContext {
        areas_specs: RwSignal::new(Default::default()),
        loaded: RwSignal::new(false),
    });
}

impl DataContext {
    pub async fn load_data(&self, backend_client: BackendClient) -> Result<(), BackendError> {
        if self.loaded.get_untracked() {
            return Ok(());
        }

        self.areas_specs
            .set(backend_client.get_areas().await?.areas);

        self.loaded.set(true);

        Ok(())
    }
}
