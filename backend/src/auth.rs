use anyhow::anyhow;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::{
    body::Body,
    extract::{Request, State},
    http::Response,
    middleware::Next,
};
use axum_extra::TypedHeader;
use chrono::{Duration, Utc};
use headers::{authorization::Bearer, Authorization};
use jsonwebtoken::{decode, encode, Header, TokenData, Validation};
use serde::{Deserialize, Serialize};

use shared::data::user::{User, UserId};

use crate::{
    app_state::{AppSettings, AppState},
    db,
    rest::AppError,
};

pub async fn verify_captcha(token: &str) -> anyhow::Result<bool> {
    // TODO: move to env!("TURNSTILE_SECRET")
    let secret = std::env::var("TURNSTILE_SECRET").expect("missing setting 'TURNSTILE_SECRET'");

    Ok(reqwest::Client::new()
        .post("https://challenges.cloudflare.com/turnstile/v0/siteverify")
        .form(&vec![("secret", secret), ("response", token.to_string())])
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?
        .get("success")
        .and_then(|success| success.as_bool())
        .unwrap_or(false))
}

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub exp: usize,  // Expiry time of the token
    pub iat: usize,  // Issued at time of the token
    pub sub: UserId, // Subject associated with the token
}
pub async fn sign_in(
    app_settings: &AppSettings,
    db_pool: &db::DbPool,
    username: &str,
    password: &str,
) -> Result<String, AppError> {
    let (user_id, password_hash_opt) = db::users::auth_user(db_pool, username)
        .await?
        .ok_or_else(|| AppError::Unauthorized("incorrect username or password".to_string()))?;

    let password_hash = password_hash_opt
        .ok_or_else(|| AppError::Unauthorized("incorrect username or password".to_string()))?;

    // TODO: Track activity logs, etc.

    if verify_password(password, &password_hash) {
        db::users::update_last_login(db_pool, &user_id)
            .await
            .unwrap_or_else(|e| tracing::error!("couldn't update user last login: {e}"));
        Ok(encode_jwt(app_settings, user_id)?)
    } else {
        Err(AppError::Unauthorized(
            "incorrect username or password".to_string(),
        ))
    }
}

#[derive(Clone)]
pub struct CurrentUser {
    pub user: User,
}

pub async fn authorization_middleware(
    State(state): State<AppState>,
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
    mut req: Request,
    next: Next,
) -> Result<Response<Body>, AppError> {
    let user_id = authorize_jwt(&state.app_settings, bearer.token())
        .ok_or_else(|| AppError::Unauthorized("invalid token".to_string()))?;

    let user = db::users::read_user(&state.db_pool, &user_id)
        .await?
        .ok_or_else(|| AppError::Unauthorized("invalid token".to_string()))?;

    req.extensions_mut()
        .insert(CurrentUser { user: user.into() });
    Ok(next.run(req).await)
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

fn encode_jwt(app_settings: &AppSettings, sub: UserId) -> anyhow::Result<String> {
    let now = Utc::now();
    let exp: usize = (now + Duration::hours(24)).timestamp() as usize;
    let iat: usize = now.timestamp() as usize;

    Ok(encode(
        &Header::default(),
        &Claims { iat, exp, sub },
        &app_settings.jwt_encoding_key,
    )?)
}

pub fn hash_password(password: &str) -> anyhow::Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    match argon2.hash_password(password.as_ref(), &salt) {
        Ok(password_hash) => Ok(password_hash.to_string()),
        Err(e) => Err(anyhow!("failed to hash password: {e}")),
    }
}

pub fn verify_password(password: &str, password_hash: &str) -> bool {
    PasswordHash::new(password_hash)
        .map(|parsed_hash| {
            Argon2::default()
                .verify_password(password.as_ref(), &parsed_hash)
                .is_ok()
        })
        .unwrap_or(false)
}

impl From<db::users::UserEntry> for User {
    fn from(val: db::users::UserEntry) -> Self {
        User {
            user_id: val.user_id,
            username: val.username.unwrap_or_default(),
            max_characters: val.max_characters as u8,
        }
    }
}
