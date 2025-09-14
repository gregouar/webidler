use shared::data::area::AreaLevel;

pub const CHARACTER_DATA_VERSION: &str = "0.1.2";

// TODO: put in some game config file
pub const DEFAULT_MAX_CHARACTERS: u8 = 5;
pub const SKILL_BASE_COST: f64 = 100.0;
pub const SKILL_COST_FACTOR: f64 = 1_000.0;
pub const CHAMPION_BASE_CHANCES: f64 = 0.0001;
pub const CHAMPION_INC_CHANCES: f64 = 0.000005;
pub const CHAMPION_LEVEL_INC: AreaLevel = 5;

pub const MONSTER_INCREASE_FACTOR: f64 = 0.125;
pub const SKILL_COST_INCREASE_FACTOR: f64 = 0.31;
pub const XP_INCREASE_FACTOR: f64 = 0.4;
pub const ARMOR_FACTOR: f64 = 100.0;

pub const MAX_ITEM_QUALITY_PER_LEVEL: f64 = 0.5;
pub const MAX_ITEM_QUALITY: f64 = 25.0;

pub const MAX_MARKET_PUBLIC_LISTINGS: i64 = 100;
pub const MAX_MARKET_PRIVATE_LISTINGS: i64 = 10;
