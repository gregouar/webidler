use serde::{Deserialize, Serialize};

pub type AreaLevel = u16;
pub type ThreatLevel = u16;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AreaSpecs {
    pub name: String,
    pub starting_level: AreaLevel,
    #[serde(default)]
    pub required_level: AreaLevel,
    #[serde(default)]
    pub starting_gold: f64,
    pub header_background: String,
    pub footer_background: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AreaState {
    pub area_level: AreaLevel,
    pub is_boss: bool,
    pub waves_done: u8,

    pub max_area_level: AreaLevel,      // Max for this grind
    pub max_area_level_ever: AreaLevel, // Max for all grind of this area
    pub last_champion_spawn: AreaLevel,

    pub auto_progress: bool,
    pub going_back: u16,

    pub end_quest: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AreaThreat {
    pub threat_level: ThreatLevel,

    pub cooldown: f32,
    pub elapsed_cooldown: f32,

    pub just_increased: bool,
}
