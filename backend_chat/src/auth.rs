use jsonwebtoken::{TokenData, Validation, decode};
use serde::{Deserialize, Serialize};

use shared::data::user::{UserDetails, UserId};

use crate::app_state::AppSettings;

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub exp: usize,  // Expiry time of the token
    pub iat: usize,  // Issued at time of the token
    pub sub: UserId, // Subject associated with the token
}

#[derive(Clone)]
pub struct CurrentUser {
    pub user_details: UserDetails,
}

pub fn authorize_jwt(app_settings: &AppSettings, token: &str) -> Option<UserId> {
    decode(
        token,
        &app_settings.jwt_decoding_key,
        &Validation::default(),
    )
    .ok()
    .map(|token_data: TokenData<Claims>| token_data.claims.sub)
}
