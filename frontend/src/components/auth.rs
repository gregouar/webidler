use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use leptos::prelude::*;
use serde::Deserialize;

use crate::components::backend_client::{BackendClient, BackendError};

const REFRESH_BEFORE_EXPIRY_SECONDS: i64 = 60;

#[derive(Clone, Copy)]
pub struct AuthState {
    jwt: RwSignal<String>,
    exp: RwSignal<Option<i64>>,
    refreshing: RwSignal<bool>,
}

impl AuthState {
    pub fn new() -> Self {
        AuthState {
            jwt: RwSignal::new(String::new()),
            exp: RwSignal::new(None),
            refreshing: RwSignal::new(true),
        }
    }

    pub fn set_access_token(&self, token: String) {
        self.exp.set(jwt_exp(&token));
        self.jwt.set(token);
        self.refreshing.set(false);
    }

    pub fn sign_out(&self) {
        self.exp.set(None);
        self.jwt.set(String::new());
        self.refreshing.set(false);
    }

    pub fn is_authenticated(&self) -> bool {
        !self.jwt.get_untracked().is_empty()
    }

    pub fn track_authenticated(&self) -> bool {
        !self.jwt.get().is_empty()
    }

    pub async fn get_access_token(&self, backend: BackendClient) -> Result<String, BackendError> {
        if !self.should_refresh() {
            return Ok(self.jwt.get_untracked());
        }

        self.refreshing.set(true);
        match backend.post_refresh().await {
            Ok(response) => {
                let token = response.jwt;
                self.set_access_token(token.clone());
                Ok(token)
            }
            Err(e) => {
                self.sign_out();
                Err(e)
            }
        }
    }

    fn should_refresh(&self) -> bool {
        let Some(exp) = self.exp.get_untracked() else {
            return true;
        };

        exp <= now_seconds() + REFRESH_BEFORE_EXPIRY_SECONDS
    }
}

#[derive(Deserialize)]
struct JwtClaims {
    exp: i64,
}

fn jwt_exp(token: &str) -> Option<i64> {
    let payload = token.split('.').nth(1)?;
    let bytes = URL_SAFE_NO_PAD.decode(payload).ok()?;
    serde_json::from_slice::<JwtClaims>(&bytes)
        .ok()
        .map(|c| c.exp)
}

fn now_seconds() -> i64 {
    (js_sys::Date::now() / 1000.0) as i64
}
