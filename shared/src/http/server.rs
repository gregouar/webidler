use chrono::{DateTime, Utc};
use std::{collections::HashMap, fmt, time::Duration};

use serde::{Deserialize, Serialize};

use crate::data::{
    area::{AreaLevel, AreaSpecs},
    skill::SkillSpecs,
    user::{User, UserCharacter, UserCharacterId, UserGrindArea},
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ErrorResponse {
    pub error: String,
}

impl fmt::Display for ErrorResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.error)
    }
}

// Stats

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

// Users

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SignUpResponse {}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SignInResponse {
    pub jwt: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GetUserResponse {
    pub user: User,
}

// Characters

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CreateCharacterResponse {
    pub character_id: UserCharacterId,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GetUserCharactersResponse {
    pub characters: Vec<UserCharacter>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GetCharacterDetailsResponse {
    pub character: UserCharacter,
    pub areas: Vec<UserGrindArea>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct DeleteCharacterResponse {}

// Game

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GetAreasResponse {
    pub areas: HashMap<String, AreaSpecs>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GetSkillsResponse {
    pub skills: HashMap<String, SkillSpecs>,
}
