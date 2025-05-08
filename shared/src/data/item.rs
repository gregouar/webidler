use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::item_affix::{AffixEffect, ItemAffix};
pub use super::skill::{Range, Shape};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum ItemRarity {
    Normal,
    Magic,
    Rare,
    Unique,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum ItemSlot {
    Amulet,
    Body,
    Boots,
    Gloves,
    Helmet,
    Relic,
    Ring,
    Shield,
    Weapon,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ItemBase {
    pub name: String,
    pub icon: String,
    pub description: String,

    pub min_level: u16,

    pub item_slot: ItemSlot,

    pub weapon_specs: Option<WeaponSpecs>,
    pub armor_specs: Option<ArmorSpecs>,
    // TODO: Implicit affixes?
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ItemSpecs {
    pub base: ItemBase,
    pub rarity: ItemRarity,

    // Area level at which the item dropped
    pub level: u16,

    pub weapon_specs: Option<WeaponSpecs>,
    pub armor_specs: Option<ArmorSpecs>,

    pub affixes: Vec<ItemAffix>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct WeaponSpecs {
    #[serde(default)]
    pub range: Range,
    #[serde(default)]
    pub shape: Shape,

    pub cooldown: f32,

    pub min_damage: f64,
    pub max_damage: f64,
    // pub critical_chances: f32,
    // pub critical_damage: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct ArmorSpecs {
    pub armor: f64,
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
