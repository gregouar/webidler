use crate::data::area::AreaLevel;

// TODO: put in some game config file
pub const DEFAULT_MAX_CHARACTERS: u8 = 5;
pub const PLAYER_LIFE_PER_LEVEL: f64 = 3.0;
pub const SKILL_BASE_COST: f64 = 100.0;
pub const SKILL_COST_FACTOR: f64 = 1_000.0;
pub const CHAMPION_BASE_CHANCE: f64 = 0.0001;
pub const CHAMPION_INC_CHANCE: f64 = 0.000005;
pub const CHAMPION_LEVEL_INC: AreaLevel = 5;

pub const WAVES_PER_AREA_LEVEL: u8 = 5;

pub const MONSTER_INCREASE_FACTOR: f64 = 0.12;
pub const SKILL_COST_INCREASE_FACTOR: f64 = 0.31;
pub const XP_INCREASE_FACTOR: f64 = 0.39;
pub const ARMOR_FACTOR: f64 = 100.0;

pub const MAX_ITEM_QUALITY_PER_LEVEL: f32 = 0.5;
pub const MAX_ITEM_QUALITY: f32 = 25.0;

pub const MAX_MARKET_PUBLIC_LISTINGS: i64 = 150;
pub const MAX_MARKET_PRIVATE_LISTINGS: i64 = 10;

pub const THREAT_COOLDOWN: f32 = 20.0;
pub const THREAT_BOSS_COOLDOWN: f32 = 60.0;
pub const THREAT_EFFECT: f64 = 0.5;
