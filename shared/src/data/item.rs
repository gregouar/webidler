use serde::{Deserialize, Serialize};

pub use super::skill::{Range, Shape};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ItemSpecs {
    pub name: String,
    pub icon: String,
    pub description: String,
    pub rarity: ItemRarity,

    // Area level at which the item dropped
    pub item_level: u16,

    pub item_category: ItemCategory,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ItemCategory {
    Trinket,
    Weapon(WeaponSpecs),
}

impl Default for ItemCategory {
    fn default() -> Self {
        ItemCategory::Trinket
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
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

    pub magic_prefixes: Vec<WeaponMagicPrefix>,
    pub magic_suffixes: Vec<WeaponMagicSuffix>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum WeaponMagicPrefix {
    AttackSpeed(f32),
    AttackDamages(f64),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum WeaponMagicSuffix {
    GoldFind(f64),
}
