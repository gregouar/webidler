use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GameStats {
    pub elapsed_time: Duration,
    pub areas_completed: u64,
    pub highest_area_level: u64,
    pub gold_collected: f64,
    pub monster_kills: u64,
    pub player_kills: u64,
}
