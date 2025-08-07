use chrono::{DateTime, Utc};
use std::{collections::HashMap, time::Duration};

use serde::{Deserialize, Serialize};

use crate::data::{skill::SkillSpecs, user::User, world::AreaLevel};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlayersCountResponse {
    pub value: i64,
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

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SignUpResponse {
    pub jwt: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SignInResponse {
    pub jwt: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GetUserResponse {
    pub user: User,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SkillsResponse {
    pub skills: HashMap<String, SkillSpecs>,
}
