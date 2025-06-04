use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use super::{
    item::ItemCategory,
    stat_effect::{Modifier, StatEffect, StatType},
    world::AreaLevel,
};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum AffixType {
    Prefix,
    Suffix,
    Unique,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum AffixTag {
    Attack,
    Armor,
    Critical,
    Fire,
    Gold,
    Life,
    Mana,
    Physical,
    Poison,
    Speed,
    Spell,
    // TODO: add others
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AffixEffectScope {
    Local,
    Global,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ItemAffixBlueprint {
    pub name: String,
    pub family: String, // Cannot generate multiple affixes of same category on item
    pub tags: HashSet<AffixTag>,

    pub affix_type: AffixType,
    pub tier: u8,
    pub weight: u64, // Bigger weight means more chances to have affix

    #[serde(default)]
    pub restrictions: Option<HashSet<ItemCategory>>,
    pub item_level: AreaLevel,

    #[serde(default)]
    pub effects: Vec<AffixEffectBlueprint>,
    // #[serde(default)]
    // pub triggers: Vec<TriggeredEffect>, // TODO
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct AffixEffectBlueprint {
    pub stat: StatType,
    pub modifier: Modifier,
    pub scope: AffixEffectScope,
    pub min: f64,
    pub max: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ItemAffix {
    pub name: String,
    pub family: String,
    pub tags: HashSet<AffixTag>,

    pub affix_type: AffixType,
    pub tier: u8,

    pub effects: Vec<AffixEffect>,
    // pub triggers: Vec<TriggeredEffect>, // TODO
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct AffixEffect {
    pub scope: AffixEffectScope,
    pub stat_effect: StatEffect,
}
