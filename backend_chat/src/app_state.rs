use axum::extract::FromRef;
use std::env;

use crate::chat::chat_state::ChatState;

#[derive(Clone)]
pub struct AppState {
    pub app_settings: AppSettings,
    pub chat_state: ChatState,
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
