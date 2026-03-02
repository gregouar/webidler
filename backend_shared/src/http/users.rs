use nutype::nutype;
use serde::{Deserialize, Serialize};

pub type UserId = uuid::Uuid;

#[nutype(
    sanitize(trim, lowercase),
    derive(Deserialize, Serialize, Debug, PartialEq, Clone, Deref)
)]
pub struct EmailNoValidate(String);

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct User {
    pub user_id: UserId,

    pub username: String,
    pub max_characters: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GetUserDetailsResponse {
    pub user_details: UserDetails,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct UserDetails {
    pub user: User,

    pub email: Option<EmailNoValidate>,
}
