use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use super::world::AreaLevel;

pub use super::effect::{DamageType, EffectModifier, EffectStat};
pub use super::skill::{Range, Shape};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum AffixType {
    Prefix,
    Suffix,
    Unique,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
// TODO: add others
pub enum AffixRestriction {
    // Major type:
    Armor,
    AttackWeapon,
    SpellWeapon,
    MeleeWeapon,
    RangedWeapon,
    Shield,
    Jewelry,
    Trinket,
    // Sub type:
    Body,
    Boots,
    Cloak,
    Gloves,
    Helmet,
    Relic,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Eq, PartialEq, Hash)]
// TODO: add others
pub enum AffixTag {
    Attack,
    Armor,
    Critical,
    Fire,
    Gold,
    Life,
    Mana,
    Physical,
    Speed,
    Spell,
}

// #[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
// pub enum AffixEffectScope {
//     Local,
//     Global,
// }

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ItemAffixBlueprint {
    pub name: String,
    pub family: String, // Cannot generate multiple affixes of same category on item
    pub tags: HashSet<AffixTag>,

    pub affix_type: AffixType,
    pub tier: u8,
    pub weight: u64, // Bigger weight means more chances to have affix

    #[serde(default)]
    pub restrictions: Option<HashSet<AffixRestriction>>,
    pub item_level: AreaLevel,

    pub effects: Vec<AffixEffectBlueprint>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct AffixEffectBlueprint {
    pub stat: EffectStat,
    pub modifier: EffectModifier,
    // pub scope: AffixEffectScope,
    pub min: f64,
    pub max: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct AffixEffect {
    pub stat: EffectStat,
    pub modifier: EffectModifier,
    pub value: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ItemAffix {
    pub name: String,
    pub family: String,
    pub tags: HashSet<AffixTag>,

    pub affix_type: AffixType,
    pub tier: u8,

    pub effects: Vec<AffixEffect>,
}
