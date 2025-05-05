use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::item_affix::{AffixEffect, ItemAffix};
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct QueuedLoot {
    pub identifier: u32,
    pub item_specs: ItemSpecs,
    pub state: LootState,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum LootState {
    Normal,
    WillDisappear,
    HasDisappeared,
}

impl Default for LootState {
    fn default() -> Self {
        LootState::Normal
    }
}
