use serde::{Deserialize, Serialize};

pub type AreaLevel = u16;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HelloSchema {
    pub greeting: String,
    pub value: i32,
}

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
    pub waves_done: u8,
}
