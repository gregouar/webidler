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
    Ok(Json(SignInResponse {
        jwt: "jwt".to_string(),
        user: User,
    }))
}
