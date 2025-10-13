use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GameStats {
    pub elapsed_time: Duration,
    pub areas_completed: u64,
    pub monsters_killed: u64,
    pub player_deaths: u64,
}
