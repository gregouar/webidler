use anyhow::anyhow;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::{body::Body, extract::Request, http, http::Response, middleware::Next};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};
use serde::{Deserialize, Serialize};

use shared::data::user::{User, UserId};

use crate::{db, rest::AppError};

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
// Define a structure for holding claims data used in JWT tokens
pub struct Claims {
    pub exp: usize,  // Expiry time of the token
    pub iat: usize,  // Issued at time of the token
    pub sub: UserId, // Subject associated with the token
}
pub async fn sign_in(
    db_pool: &db::DbPool,
    username: &str,
    password: &str,
) -> Result<String, AppError> {
    let (user_id, password_hash_opt) = db::users::auth_user(db_pool, username)
        .await?
        .ok_or_else(|| AppError::Unauthorized("incorrect username or password".to_string()))?;

    let password_hash = password_hash_opt
        .ok_or_else(|| AppError::Unauthorized("incorrect username or password".to_string()))?;

    // TODO: Track last login, activity logs, etc.

    if verify_password(password, &password_hash) {
        Ok(encode_jwt(user_id)?)
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
    mut req: Request,
    next: Next,
) -> Result<Response<Body>, AppError> {
    let user_id = req
        .headers_mut()
        .get(http::header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .and_then(|header| header.strip_prefix("Bearer "))
        .and_then(|token| authorize_jwt(token))
        .ok_or_else(|| AppError::Unauthorized("invalid token".to_string()))?;

    let current_user = get_user(&user_id)
        .await
        .ok_or_else(|| AppError::Unauthorized("invalid token".to_string()))?;

    req.extensions_mut().insert(current_user);
    Ok(next.run(req).await)
}

pub fn authorize_jwt(token: &str) -> Option<UserId> {
    decode_jwt(token)
        .ok()
        .map(|token_data| token_data.claims.sub)
}

async fn get_user(user_id: &UserId) -> Option<CurrentUser> {
    let current_user: CurrentUser = CurrentUser {
        user: User {
            user_id: user_id.clone(),
            username: "username".to_string(),
            max_characters: 5,
        },
    };
    Some(current_user)
}

fn encode_jwt(sub: UserId) -> anyhow::Result<String> {
    let now = Utc::now();
    let exp: usize = (now + Duration::hours(24)).timestamp() as usize;
    let iat: usize = now.timestamp() as usize;

    Ok(encode(
        &Header::default(),
        &Claims { iat, exp, sub },
        &EncodingKey::from_secret(env!("JWT_SECRET").as_ref()),
    )?)
}

fn decode_jwt(jwt_token: &str) -> anyhow::Result<TokenData<Claims>> {
    Ok(decode(
        jwt_token,
        &DecodingKey::from_secret(env!("JWT_SECRET").as_ref()),
        &Validation::default(),
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
    PasswordHash::new(&password_hash)
        .map(|parsed_hash| {
            Argon2::default()
                .verify_password(password.as_ref(), &parsed_hash)
                .is_ok()
        })
        .unwrap_or(false)
}
