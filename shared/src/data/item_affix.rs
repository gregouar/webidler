use serde::{Deserialize, Serialize};

use super::item::ItemSlot;
pub use super::skill::{Range, Shape};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ItemStat {
    AttackSpeed,
    AttackDamage,
    MinAttackDamage,
    MaxAttackDamage,
    Armor,
    GoldFind,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum AffixType {
    Prefix,
    Suffix,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AffixEffectType {
    Flat,
    Multiplier,
}

// #[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
// pub enum AffixEffectScope {
//     Local,
//     Global,
// }

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct AffixEffectBlueprint {
    pub stat: ItemStat,
    pub effect_type: AffixEffectType,
    // pub scope: AffixEffectScope,
    pub min_value: f64,
    pub max_value: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ItemAffixBlueprint {
    pub name: String,
    pub family: String, // Cannot generate multiple affixes of same category on item

    pub affix_type: AffixType,
    pub affix_level: u8,
    pub weight: f32, // Bigger weight means more chances to have affix

    pub allowed_items: Vec<ItemSlot>,
    pub min_item_level: u8,

    pub effects: Vec<AffixEffectBlueprint>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct AffixEffect {
    pub stat: ItemStat,
    pub effect_type: AffixEffectType,
    pub value: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ItemAffix {
    pub name: String,
    pub family: String,

    pub affix_type: AffixType,
    pub affix_level: u8,

    pub effects: Vec<AffixEffect>,
}
