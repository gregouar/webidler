use serde::{Deserialize, Serialize};

use crate::data::area::AreaLevel;

pub type UserId = uuid::Uuid;
pub type UserCharacterId = uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct User {
    pub user_id: UserId,

    pub username: String,
    pub max_characters: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserCharacter {
    pub character_id: UserCharacterId,

    pub name: String,
    pub portrait: String,
    pub max_area_level: AreaLevel,
    // pub resources...,
}
