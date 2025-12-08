use serde::{Deserialize, Serialize};

use crate::{
    data::area::{AreaLevel, AreaSpecs},
    types::Email,
};

pub type UserId = uuid::Uuid;
pub type UserCharacterId = uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct User {
    pub user_id: UserId,

    pub username: String,
    pub max_characters: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct UserDetails {
    pub user: User,

    pub email: Option<Email>,
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
    pub character_id: UserCharacterId,

    pub name: String,
    pub portrait: String,
    pub max_area_level: AreaLevel,

    pub resource_gems: f64,
    pub resource_shards: f64,
    pub resource_gold: f64,

    pub activity: UserCharacterActivity,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct UserGrindArea {
    pub area_id: String,
    pub area_specs: AreaSpecs,
    pub max_level_reached: AreaLevel,
}
