use leptos::prelude::*;

#[derive(Clone)]
pub struct TownContext {
    // TODO: Is this really the correct place? Should we have a UI context?
    // TODO: enum ?
    pub open_ascend: RwSignal<bool>,
    // TODO: Add inventory, ascendance, etc?
}

impl Default for TownContext {
    fn default() -> Self {
        Self::new()
    }
}

impl TownContext {
    pub fn new() -> Self {
        TownContext {
            open_ascend: RwSignal::new(false),
        }
    }
}
