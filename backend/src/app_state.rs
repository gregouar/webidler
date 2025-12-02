use aes_gcm::{Aes256Gcm, KeyInit};
use axum::extract::FromRef;
use base64::prelude::*;
use jsonwebtoken::{DecodingKey, EncodingKey};
use std::env;

use crate::integration::discord::DiscordState;
pub use crate::{
    db::pool::DbPool,
    email::EmailService,
    game::{data::master_store::MasterStore, sessions::SessionsStore},
};

#[derive(Clone)]
pub struct AppState {
    pub app_settings: AppSettings,
    pub db_pool: DbPool,
    pub email_service: EmailService,
    pub master_store: MasterStore,
    pub sessions_store: SessionsStore,
    pub discord_state: DiscordState,
}

#[derive(Clone)]
pub struct AppSettings {
    pub jwt_encoding_key: EncodingKey,
    pub jwt_decoding_key: DecodingKey,
    pub aes_key: Aes256Gcm,
    pub hash_key: String,
    pub frontend_url: String,
}

impl AppSettings {
    pub fn from_env() -> Self {
        let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
        let aes_key_str: [u8; 32] = BASE64_STANDARD
            .decode(
                std::env::var("AES_KEY")
                    .expect("AES_KEY must be set")
                    .as_bytes(),
            )
            .expect("AES_KEY must be base64")
            .try_into()
            .expect("AES_KEY must be 32 bytes");

        Self {
            jwt_encoding_key: EncodingKey::from_secret(jwt_secret.as_ref()),
            jwt_decoding_key: DecodingKey::from_secret(jwt_secret.as_ref()),
            aes_key: Aes256Gcm::new_from_slice(&aes_key_str).expect("failed to create AES key"),
            hash_key: env::var("HASH_KEY").expect("HASH_KEY must be set"),
            frontend_url: env::var("FRONTEND_URL").expect("FRONTEND_URL must be set"),
        }
    }
}

impl FromRef<AppState> for AppSettings {
    fn from_ref(app_state: &AppState) -> AppSettings {
        app_state.app_settings.clone()
    }
}
impl FromRef<AppState> for DbPool {
    fn from_ref(app_state: &AppState) -> DbPool {
        app_state.db_pool.clone()
    }
}
impl FromRef<AppState> for EmailService {
    fn from_ref(app_state: &AppState) -> EmailService {
        app_state.email_service.clone()
    }
}
impl FromRef<AppState> for MasterStore {
    fn from_ref(app_state: &AppState) -> MasterStore {
        app_state.master_store.clone()
    }
}
impl FromRef<AppState> for SessionsStore {
    fn from_ref(app_state: &AppState) -> SessionsStore {
        app_state.sessions_store.clone()
    }
}
impl FromRef<AppState> for DiscordState {
    fn from_ref(app_state: &AppState) -> DiscordState {
        app_state.discord_state.clone()
    }
}
