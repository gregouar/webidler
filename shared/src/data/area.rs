use serde::{Deserialize, Serialize};

use crate::{
    constants::{ITEM_REWARD_MIN_PICKS, ITEM_REWARD_MIN_SLOTS},
    data::{
        modifier::ModifiableValue,
        stat_effect::EffectsMap,
        trigger::TriggeredEffect,
        values::{Cooldown, NonNegative},
    },
};

pub type AreaLevel = u16;
pub type ThreatLevel = u16;

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct AreaSpecs {
    pub name: String,
    pub description: String,
    pub header_background: String,
    pub footer_background: String,

    pub power_level: AreaLevel,
    #[serde(default)]
    pub required_level: AreaLevel,
    #[serde(default)]
    pub item_level_modifier: ModifiableValue<AreaLevel>,

    #[serde(default)]
    pub coming_soon: bool,
    #[serde(default)]
    pub disable_shards: bool,

    #[serde(default = "default_reward_slots")]
    pub reward_slots: u8,
    #[serde(default = "default_reward_picks")]
    pub reward_picks: u8,
    #[serde(default = "default_item_rarity")]
    pub loot_rarity: ModifiableValue<f64>,
    #[serde(default = "default_item_rarity")]
    pub gems_find: ModifiableValue<f64>,

    #[serde(default)]
    pub effects: EffectsMap,
    #[serde(default)]
    pub triggers: Vec<TriggeredEffect>,
}

fn default_reward_slots() -> u8 {
    ITEM_REWARD_MIN_SLOTS
}

fn default_reward_picks() -> u8 {
    ITEM_REWARD_MIN_PICKS
}

fn default_item_rarity() -> ModifiableValue<f64> {
    100.0.into()
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AreaState {
    pub area_level: AreaLevel,
    pub is_boss: bool,
    pub waves_done: u8, // TODO: could rename to current wave

    pub max_area_level: AreaLevel,      // Max for this grind
    pub max_area_level_ever: AreaLevel, // Max for all grind of this area
    pub last_champion_spawn: AreaLevel,

    pub auto_progress: bool,
    pub going_back: u16,
    pub rush_mode: bool,
    // pub end_quest: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AreaThreat {
    pub threat_level: ThreatLevel,

    pub cooldown: NonNegative,
    pub elapsed_cooldown: Cooldown,

    pub just_increased: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct StartAreaConfig {
    pub area_id: String,
    pub map_item_index: Option<u8>,
}
