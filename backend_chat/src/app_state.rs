use axum::extract::FromRef;
use jsonwebtoken::{DecodingKey, EncodingKey};
use std::env;

pub use crate::db::pool::DbPool;

#[derive(Clone)]
pub struct AppState {
    pub app_settings: AppSettings,
    // TODO: Banned, Muted, SpamBucket in some Moderation thingy?
}

#[derive(Clone)]
pub struct AppSettings {
    pub backend_url: String,
}

impl AppSettings {
    pub fn from_env() -> Self {
        Self {
            backend_url: env::var("BACKEND_URL").expect("BACKEND_URL must be set"),
        }
    }
}

impl FromRef<AppState> for AppSettings {
    fn from_ref(app_state: &AppState) -> AppSettings {
        app_state.app_settings.clone()
    }
}
