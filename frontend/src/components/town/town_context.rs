use leptos::prelude::*;
use shared::data::{
    passive::{PassivesTreeAscension, PassivesTreeSpecs},
    user::{UserCharacter, UserGrindArea},
};

#[derive(Clone, Copy)]
pub struct TownContext {
    pub character: RwSignal<UserCharacter>,
    pub areas: RwSignal<Vec<UserGrindArea>>,
    // TODO: Add inventory, ascendance, etc?
    pub passives_tree_specs: RwSignal<PassivesTreeSpecs>,
    pub passives_tree_ascension: RwSignal<PassivesTreeAscension>,

    // TODO: Is this really the correct place? Should we have a UI context?
    // TODO: enum ?
    pub open_ascend: RwSignal<bool>,
    pub open_market: RwSignal<bool>,
}

impl Default for TownContext {
    fn default() -> Self {
        Self::new()
    }
}

impl TownContext {
    pub fn new() -> Self {
        TownContext {
            character: RwSignal::new(Default::default()),
            areas: RwSignal::new(Vec::new()),
            passives_tree_specs: RwSignal::new(Default::default()),
            passives_tree_ascension: RwSignal::new(Default::default()),
            open_ascend: RwSignal::new(false),
            open_market: RwSignal::new(false),
        }
    }
}
