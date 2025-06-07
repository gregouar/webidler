use chrono::{DateTime, Utc};
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::data::world::AreaLevel;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlayersCountResponse {
    pub value: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct LeaderboardResponse {
    pub entries: Vec<LeaderboardEntry>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LeaderboardEntry {
    pub player_name: String,
    pub area_level: AreaLevel,
    pub time_played: Duration,
    pub created_at: DateTime<Utc>,
    pub comments: String,
}
