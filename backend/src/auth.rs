use aes_gcm::aead::Aead;
use aes_gcm::Nonce;
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
use base64::{
    alphabet,
    engine::{self, general_purpose},
    Engine as _,
};
use chrono::{Duration, Utc};
use headers::{authorization::Bearer, Authorization};
use jsonwebtoken::{decode, encode, Header, TokenData, Validation};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use shared::{
    data::user::{User, UserDetails, UserId},
    types::Email,
};

use crate::{
    app_state::{AppSettings, AppState},
    db,
    rest::AppError,
};

const B64_ENGINE: engine::GeneralPurpose =
    engine::GeneralPurpose::new(&alphabet::URL_SAFE, general_purpose::NO_PAD);

pub async fn verify_captcha(token: &str) -> anyhow::Result<bool> {
    // TODO: move to app_settings
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
    pub user_details: UserDetails,
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

    let email = user
        .email_crypt
        .as_ref()
        .and_then(|email_crypt| decrypt_email(&state.app_settings, email_crypt).ok());

    req.extensions_mut().insert(CurrentUser {
        user_details: UserDetails {
            email,
            user: user.into(),
        },
    });
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
    match Argon2::default().hash_password(password.as_ref(), &SaltString::generate(&mut OsRng)) {
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

pub fn encrypt_email(app_settings: &AppSettings, email: &str) -> anyhow::Result<Vec<u8>> {
    let mut nonce = [0u8; 12];
    rand::rng().fill_bytes(&mut nonce);

    let ciphertext = app_settings
        .aes_key
        .encrypt(Nonce::from_slice(&nonce), email.as_bytes())
        .map_err(|_| anyhow!("failed to encrypt"))?;

    let mut combined = Vec::with_capacity(nonce.len() + ciphertext.len());
    combined.extend_from_slice(&nonce);
    combined.extend_from_slice(&ciphertext);

    Ok(combined)
}

pub fn decrypt_email(app_settings: &AppSettings, data: &[u8]) -> anyhow::Result<Email> {
    let (nonce_bytes, ciphertext) = data.split_at_checked(12).ok_or(anyhow!("invalid data"))?;

    Ok(Email::try_new(String::from_utf8(
        app_settings
            .aes_key
            .decrypt(Nonce::from_slice(nonce_bytes), ciphertext)
            .map_err(|_| anyhow!("failed to decrypt"))?,
    )?)?)
}

pub fn hash_content(app_settings: &AppSettings, email: &str) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(&app_settings.hash_key);
    hasher.update(email.as_bytes());
    hasher.finalize().to_vec()
}

pub fn generate_token() -> String {
    let mut token_data = [0u8; 32];
    rand::rng().fill_bytes(&mut token_data);

    B64_ENGINE.encode(token_data)
}
