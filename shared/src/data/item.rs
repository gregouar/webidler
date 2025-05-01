use std::collections::HashMap;

use serde::{Deserialize, Serialize};

pub use super::skill::{Range, Shape};

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct ItemSpecs {
    pub name: String,
    pub icon: String,
    pub description: String,
    pub rarity: ItemRarity,

    // Area level at which the item dropped
    pub item_level: u16,

    pub item_category: ItemCategory,

    pub affixes: Vec<ItemAffix>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum ItemRarity {
    Normal,
    Magic,
    Rare,
    Unique,
}

impl Default for ItemRarity {
    fn default() -> Self {
        ItemRarity::Normal
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ItemCategory {
    Trinket,
    Weapon(WeaponSpecs),
}

impl Default for ItemCategory {
    fn default() -> Self {
        ItemCategory::Trinket
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct WeaponSpecs {
    pub base_cooldown: f32,
    pub cooldown: f32,

    #[serde(default)]
    pub range: Range,
    #[serde(default)]
    pub shape: Shape,

    pub base_min_damage: f64,
    pub min_damage: f64,
    pub base_max_damage: f64,
    pub max_damage: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ItemStat {
    AttackSpeed,
    AttackDamage,
    MinAttackDamage,
    MaxAttackDamage,
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

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct AffixEffectBlueprint {
    pub stat: ItemStat,
    pub effect_type: AffixEffectType,
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

    pub allowed_items: Vec<ItemCategory>,
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

impl ItemSpecs {
    pub fn aggregate_effects(&self) -> Vec<AffixEffect> {
        self.affixes
            .iter()
            .flat_map(|affix| affix.effects.iter())
            .fold(HashMap::new(), |mut effects_map, effect| {
                *effects_map
                    .entry((effect.stat, effect.effect_type))
                    .or_default() += effect.value;
                effects_map
            })
            .into_iter()
            .map(|((stat, effect_type), value)| AffixEffect {
                stat,
                effect_type,
                value,
            })
            .collect()
    }
}
