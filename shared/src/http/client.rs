use serde::{Deserialize, Serialize};

use crate::{
    data::{passive::PassivesTreeState, user::UserCharacterId},
    types::{AssetName, Email, Password, Username},
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SignUpRequest {
    pub captcha_token: String,

    pub username: Username,
    pub email: Option<Email>,
    pub password: Password,
    pub accepted_terms: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SignInRequest {
    pub captcha_token: String,

    pub username: Username,
    pub password: Password,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateCharacterRequest {
    pub name: Username,
    pub portrait: AssetName,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AscendPassivesRequest {
    pub character_id: UserCharacterId,
    pub passives_tree_state: PassivesTreeState,
}
