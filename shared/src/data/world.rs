use serde::{Deserialize, Serialize};

pub type AreaLevel = u16;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct WorldSpecs {
    pub name: String,
    pub starting_level: AreaLevel,
    pub musics: Vec<String>,
    pub header_background: String,
    pub footer_background: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct WorldState {
    pub area_level: AreaLevel,
    pub is_boss: bool,
    pub waves_done: u8,

    pub auto_progress: bool,
    pub going_back: u16,

    pub end_quest: bool,
}
