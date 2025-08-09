use serde::{Deserialize, Serialize};

use crate::data::area::AreaLevel;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct User {
    pub user_id: String,
    pub username: String,
    pub max_characters: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserCharacter {
    pub character_id: String,
    pub name: String,
    pub portrait: String,
    pub max_area_level: AreaLevel,
    // pub resources...,
}
