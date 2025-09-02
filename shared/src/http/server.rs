use chrono::{DateTime, Utc};
use std::{collections::HashMap, fmt};

use serde::{Deserialize, Serialize};

use crate::data::{
    area::{AreaLevel, AreaSpecs},
    market::MarketItem,
    passive::{PassivesTreeAscension, PassivesTreeSpecs},
    player::PlayerInventory,
    skill::SkillSpecs,
    user::{User, UserCharacter, UserCharacterId, UserGrindArea, UserId},
};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
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
    pub user_id: UserId,
    pub username: String,
    pub character_id: UserCharacterId,
    pub character_name: String,

    pub area_id: String,
    pub area_level: AreaLevel,
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
    pub inventory: PlayerInventory,
    pub ascension: PassivesTreeAscension,
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

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GetPassivesResponse {
    pub passives_tree_specs: PassivesTreeSpecs,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AscendPassivesResponse {
    pub character: UserCharacter,
    pub ascension: PassivesTreeAscension,
}

// Market

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct BrowseMarketItemsResponse {
    pub items: Vec<MarketItem>,
    pub max_items: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SellMarketItemResponse {
    pub inventory: PlayerInventory,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BuyMarketItemResponse {
    pub character: UserCharacter,
    pub inventory: PlayerInventory,
}
