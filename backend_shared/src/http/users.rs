use nutype::nutype;
use serde::{Deserialize, Serialize};

pub type UserId = uuid::Uuid;
pub type CharacterId = uuid::Uuid;

#[nutype(
    sanitize(trim, lowercase),
    derive(Deserialize, Serialize, Debug, PartialEq, Clone, Deref)
)]
pub struct EmailNoValidate(String);

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct User {
    pub user_id: UserId,
    pub username: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct UserDetails {
    pub user: User,
    pub email: Option<EmailNoValidate>,
    pub max_characters: u8,
    pub chat_badge: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GetUserDetailsResponse {
    pub user_details: UserDetails,
}

#[derive(Deserialize)]
pub struct GetUserCharactersResponse {
    pub characters: Vec<UserCharacter>,
}

#[derive(Deserialize)]
pub struct UserCharacter {
    pub character_id: CharacterId,
    pub name: String,
    pub cosmetics: CharacterCosmetics,
}

#[derive(Deserialize, Default, Clone)]
pub struct CharacterCosmetics {
    pub title: Option<String>,
    pub badge: Option<String>,
}
