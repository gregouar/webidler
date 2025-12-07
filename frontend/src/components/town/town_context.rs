use std::collections::HashMap;

use leptos::prelude::*;

use shared::data::{
    game_stats::GrindStats,
    passive::{PassivesTreeAscension, PassivesTreeSpecs},
    player::PlayerInventory,
    stash::Stash,
    temple::{BenedictionSpecs, PlayerBenedictions},
    user::{UserCharacter, UserGrindArea},
};

#[derive(Clone, Copy, Default)]
pub struct TownContext {
    pub character: RwSignal<UserCharacter>,
    pub areas: RwSignal<Vec<UserGrindArea>>,
    pub inventory: RwSignal<PlayerInventory>,

    pub user_stash: RwSignal<Stash>,
    pub market_stash: RwSignal<Stash>,

    pub passives_tree_specs: RwSignal<PassivesTreeSpecs>,
    pub passives_tree_ascension: RwSignal<PassivesTreeAscension>,

    pub benedictions_specs: RwSignal<HashMap<String, BenedictionSpecs>>,
    pub player_benedictions: RwSignal<PlayerBenedictions>,

    pub last_grind: RwSignal<Option<GrindStats>>,

    // TODO: Is this really the correct place? Should we have a UI context?
    // TODO: enum ?
    pub open_inventory: RwSignal<bool>,
    pub open_stash: RwSignal<bool>,
    pub open_ascend: RwSignal<bool>,
    pub open_market: RwSignal<bool>,
    pub open_forge: RwSignal<bool>,
    pub open_temple: RwSignal<bool>,
}

// impl Default for TownContext {
//     fn default() -> Self {
//         Self::new()
//     }
// }

// impl TownContext {
//     pub fn new() -> Self {
//         TownContext {
//             character: RwSignal::new(Default::default()),
//             areas: RwSignal::new(Vec::new()),
//             inventory: RwSignal::new(Default::default()),
//             passives_tree_specs: RwSignal::new(Default::default()),
//             passives_tree_ascension: RwSignal::new(Default::default()),
//             open_ascend: RwSignal::new(false),
//             open_market: RwSignal::new(false),
//         }
//     }
// }
