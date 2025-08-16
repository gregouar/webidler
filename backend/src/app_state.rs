use axum::extract::FromRef;
use jsonwebtoken::{DecodingKey, EncodingKey};
use std::env;

pub use crate::{
    db::pool::DbPool,
    game::{data::master_store::MasterStore, sessions::SessionsStore},
};

#[derive(Clone)]
pub struct AppState {
    pub app_settings: AppSettings,
    pub db_pool: DbPool,
    pub master_store: MasterStore,
    pub sessions_store: SessionsStore,
}

#[derive(Clone)]
pub struct AppSettings {
    pub jwt_encoding_key: EncodingKey,
    pub jwt_decoding_key: DecodingKey,
}

impl AppSettings {
    pub fn from_env() -> Self {
        let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");

        Self {
            jwt_encoding_key: EncodingKey::from_secret(jwt_secret.as_ref()),
            jwt_decoding_key: DecodingKey::from_secret(jwt_secret.as_ref()),
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
