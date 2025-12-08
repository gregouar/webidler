use crate::data::{area::AreaLevel, stash::StashPrice};

// TODO: put in some game config file
pub const DEFAULT_MAX_CHARACTERS: u8 = 5;
pub const PLAYER_LIFE_PER_LEVEL: f64 = 3.0;
pub const SKILL_BASE_COST: f64 = 100.0;
pub const SKILL_COST_FACTOR: f64 = 1_000.0;
pub const CHAMPION_BASE_CHANCE: f64 = 0.0001;
pub const CHAMPION_INC_CHANCE: f64 = 0.000005;
pub const CHAMPION_LEVEL_INC: AreaLevel = 5;

pub const WAVES_PER_AREA_LEVEL: u8 = 5;

pub const MONSTERS_DEFAULT_DAMAGE_INCREASE: f64 = 5.0;
pub const MONSTER_INCREASE_FACTOR: f64 = 0.12;
pub const SKILL_COST_INCREASE_FACTOR: f64 = 0.31;
pub const XP_INCREASE_FACTOR: f64 = 0.39;
pub const ARMOR_FACTOR: f64 = 100.0;

pub const MAX_ITEM_QUALITY_PER_LEVEL: f32 = 0.5;
pub const MAX_ITEM_QUALITY: f32 = 25.0;

pub const THREAT_COOLDOWN: f32 = 20.0;
pub const THREAT_BOSS_COOLDOWN: f32 = 60.0;
pub const THREAT_EFFECT: f64 = 0.5;

pub const STASH_USER_PRICE: StashPrice = StashPrice {
    start_price: 1e5,
    start_size: 40,
    upgrade_price: 1e5,
    upgrade_size: 10,
};

pub const STASH_MARKET_PRICE: StashPrice = StashPrice {
    start_price: 1e7,
    start_size: 20,
    upgrade_price: 1e5,
    upgrade_size: 10,
};
