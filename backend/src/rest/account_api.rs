use axum::{extract::State, routing::post, Json, Router};

use shared::{
    data::user::User,
    http::{
        client::{SignInRequest, SignUpRequest},
        server::{SignInResponse, SignUpResponse},
    },
};

use crate::{app_state::AppState, db};

use super::AppError;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/account/signup", post(post_signup))
        .route("/account/signin", post(post_signin))
}

async fn post_signup(
    State(db_pool): State<db::DbPool>,
    Json(payload): Json<SignUpRequest>,
) -> Result<Json<SignUpResponse>, AppError> {
    Ok(Json(SignUpResponse {
        jwt: "jwt".to_string(),
    }))
}

async fn post_signin(
    State(db_pool): State<db::DbPool>,
    Json(payload): Json<SignInRequest>,
) -> Result<Json<SignInResponse>, AppError> {
    if !verify_captcha(&payload.captcha_token).await {
        return Err(AppError::NotFound);
        // TODO: give proper error
    }

    Ok(Json(SignInResponse {
        jwt: "jwt".to_string(),
    }))
}

async fn verify_captcha(token: &str) -> bool {
    let secret = std::env::var("TURNSTILE_SECRET").unwrap();
    let client = reqwest::Client::new();
    let params = vec![("secret", secret), ("response", token.to_string())];

    // if let Some(ip) = user_ip {
    //     params.push(("remoteip", ip.to_string()));
    // }

    if let Ok(res) = client
        .post("https://challenges.cloudflare.com/turnstile/v0/siteverify")
        .form(&params)
        .send()
        .await
    {
        res.json::<serde_json::Value>()
            .await
            .ok()
            .and_then(|json| json.get("success").and_then(|s| s.as_bool()))
            .unwrap_or(false)
    } else {
        false
    }
}
