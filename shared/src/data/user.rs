use serde::{Deserialize, Serialize};

use crate::{
    data::{area::AreaLevel, realms::Realm},
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
    pub max_area_level: AreaLevel,

    pub is_ssf: bool,

    pub resource_gems: f64,
    pub resource_shards: f64,
    pub resource_gold: f64,

    pub activity: UserCharacterActivity,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct UserGrindArea {
    pub area_id: String,
    pub max_level_reached: AreaLevel,
}
