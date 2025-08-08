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
    verify_captcha(&payload.captcha_token).await?;

    Ok(Json(SignUpResponse {
        success: true,
        reason: None,
    }))
}

async fn post_signin(
    State(db_pool): State<db::DbPool>,
    Json(payload): Json<SignInRequest>,
) -> Result<Json<SignInResponse>, AppError> {
    verify_captcha(&payload.captcha_token).await?;

    Ok(Json(SignInResponse {
        user_id: payload.username, //TODO
        jwt: "jwt".to_string(),    //TODO
    }))
}

async fn verify_captcha(token: &str) -> anyhow::Result<bool> {
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
