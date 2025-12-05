use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

use crate::{
    data::{
        area::AreaLevel,
        item::{ItemCategory, ItemRarity, ItemSpecs},
        passive::StatEffect,
        user::UserCharacterId,
    },
    types::{ItemName, ItemPrice},
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MarketItem {
    pub item_id: usize,

    pub owner_id: UserCharacterId,
    pub owner_name: String,

    pub recipient: Option<(UserCharacterId, String)>,
    pub rejected: bool,

    pub price: f64,

    pub item_specs: ItemSpecs,

    pub created_at: DateTime<Utc>,

    pub deleted_at: Option<DateTime<Utc>>,
    pub deleted_by: Option<(UserCharacterId, String)>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StashItem {
    pub item_id: usize,

    pub character_id: UserCharacterId,
    pub character_name: String,

    pub item_specs: ItemSpecs,

    pub created_at: DateTime<Utc>,
}

pub const STAT_FILTERS_AMOUNT: usize = 5;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct MarketFilters {
    pub order_by: MarketOrderBy,

    pub item_name: Option<ItemName>,
    pub item_level: Option<AreaLevel>,
    pub price: Option<ItemPrice>,

    pub item_rarity: Option<ItemRarity>,
    pub item_category: Option<ItemCategory>,

    pub item_damages: Option<f64>,
    pub item_damage_physical: Option<f64>,
    pub item_damage_fire: Option<f64>,
    pub item_damage_poison: Option<f64>,
    pub item_damage_storm: Option<f64>,
    pub item_crit_chance: Option<f64>,
    pub item_crit_damage: Option<f64>,
    pub item_armor: Option<f64>,
    pub item_block: Option<f64>,

    pub stat_filters: [Option<StatEffect>; STAT_FILTERS_AMOUNT],
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default, EnumIter, Hash, PartialEq, Eq)]
pub enum MarketOrderBy {
    Price,
    StatFilters,
    Damage,
    DamagePhysical,
    DamageFire,
    DamagePoison,
    DamageStorm,
    CritChance,
    CritDamage,
    Armor,
    Block,
    Level,
    #[default]
    Time,
}
