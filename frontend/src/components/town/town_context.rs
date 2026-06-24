use indexmap::IndexMap;
use leptos::prelude::*;
use std::collections::HashMap;

use shared::data::{
    passive::{PassivesTreeAscension, PassivesTreeSpecs, PurchasedNodes},
    player::PlayerInventory,
    skill::SkillSpecs,
    skill_mastery::PlayerSkillMasteries,
    stash::{Stash, StashType},
    temple::{BenedictionsCategory, PlayerBenedictions},
    user::{UserCharacter, UserGrindArea},
};

use crate::components::shared::inventory::InventoryEquipFilter;

#[derive(Clone, Copy)]
pub struct TownContext {
    pub character: RwSignal<UserCharacter>,
    pub areas: RwSignal<Vec<UserGrindArea>>,
    pub inventory: RwSignal<PlayerInventory>,

    pub character_stash: RwSignal<Stash>,
    pub user_stash: RwSignal<Stash>,
    pub market_stash: RwSignal<Stash>,

    pub passives_tree_specs: RwSignal<PassivesTreeSpecs>,
    pub passives_tree_ascension: RwSignal<PassivesTreeAscension>,
    pub passives_tree_build: RwSignal<PurchasedNodes>,

    pub benedictions_specs: RwSignal<IndexMap<String, BenedictionsCategory>>,
    pub player_benedictions: RwSignal<PlayerBenedictions>,

    pub player_skill_masteries: RwSignal<PlayerSkillMasteries>,
    pub skill_mastery_skill_specs: RwSignal<HashMap<String, SkillSpecs>>,
    pub selected_skill_mastery: RwSignal<Option<String>>,

    // pub last_grind: RwSignal<Option<GrindStats>>,
    pub selected_item_index: RwSignal<Option<u8>>,
    pub equip_filter: RwSignal<InventoryEquipFilter>,

    // TODO: Is this really the correct place? Should we have a UI context?
    // TODO: enum ?
    pub open_inventory: RwSignal<bool>,
    pub open_stash: RwSignal<bool>,
    pub open_ascend: RwSignal<bool>,
    pub open_market: RwSignal<bool>,
    pub open_forge: RwSignal<bool>,
    pub open_temple: RwSignal<bool>,
    pub open_skill_masteries: RwSignal<bool>,
    pub open_skill_mastery_details: RwSignal<bool>,
    pub open_settings: RwSignal<bool>,
}

impl Default for TownContext {
    fn default() -> Self {
        Self {
            character: Default::default(),
            areas: Default::default(),
            inventory: Default::default(),
            character_stash: RwSignal::new(Stash {
                stash_type: StashType::Character,
                ..Default::default()
            }),
            user_stash: RwSignal::new(Stash {
                stash_type: StashType::User,
                ..Default::default()
            }),
            market_stash: RwSignal::new(Stash {
                stash_type: StashType::Market,
                ..Default::default()
            }),
            passives_tree_specs: Default::default(),
            passives_tree_ascension: Default::default(),
            passives_tree_build: Default::default(),
            benedictions_specs: Default::default(),
            player_benedictions: Default::default(),
            player_skill_masteries: Default::default(),
            skill_mastery_skill_specs: Default::default(),
            selected_skill_mastery: Default::default(),
            // last_grind: Default::default(),
            selected_item_index: Default::default(),
            equip_filter: Default::default(),
            open_inventory: Default::default(),
            open_stash: Default::default(),
            open_ascend: Default::default(),
            open_market: Default::default(),
            open_forge: Default::default(),
            open_temple: Default::default(),
            open_skill_masteries: Default::default(),
            open_skill_mastery_details: Default::default(),
            open_settings: Default::default(),
        }
    }
}
