use leptos::prelude::*;
use shared::data::user::{UserCharacter, UserGrindArea};

#[derive(Clone)]
pub struct TownContext {
    pub character: RwSignal<UserCharacter>,
    pub areas: RwSignal<Vec<UserGrindArea>>,
    // TODO: Add inventory, ascendance, etc?

    // TODO: Is this really the correct place? Should we have a UI context?
    // TODO: enum ?
    pub open_ascend: RwSignal<bool>,
}

impl Default for TownContext {
    fn default() -> Self {
        Self::new()
    }
}

impl TownContext {
    pub fn new() -> Self {
        TownContext {
            character: RwSignal::new(UserCharacter::default()),
            areas: RwSignal::new(Vec::new()),
            open_ascend: RwSignal::new(false),
        }
    }
}
