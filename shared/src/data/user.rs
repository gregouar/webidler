use std::{
    collections::{HashMap, HashSet},
    time::Duration,
};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    data::{area::AreaLevel, cosmetics::CharacterCosmetics, realms::Realm},
    types::Email,
};

pub type UserId = uuid::Uuid;
pub type UserCharacterId = uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct User {
    pub user_id: UserId,
    pub username: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct UserDetails {
    pub user: User,
    pub email: Option<Email>,
    pub chat_badge: Option<String>,
    pub max_characters: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub enum UserCharacterActivity {
    #[default]
    Rusting,
    Grinding(String, AreaLevel),
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct UserCharacter {
    pub user_id: UserId,
    pub realm: Realm,

    pub character_id: UserCharacterId,

    pub name: String,
    pub portrait: String,
    pub cosmetics: CharacterCosmetics,
    pub max_area_level: AreaLevel,

    pub is_ssf: bool,

    pub resource_gems: f64,
    pub resource_shards: f64,
    pub resource_gold: f64,
    pub resource_stamina: Duration,

    pub activity: UserCharacterActivity,
    pub played_time: Duration,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct UserGrindArea {
    pub area_id: String,
    pub max_level_reached: AreaLevel,
    pub max_power_shard_level: AreaLevel,
    pub quest_completed: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct UserUnlocks {
    pub achievements: HashMap<String, DateTime<Utc>>,
    pub cosmetics: HashSet<String>,
    pub pets: HashSet<String>,
}
