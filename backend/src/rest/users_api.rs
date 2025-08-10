use axum::{
    extract::State,
    middleware,
    routing::{get, post},
    Extension, Json, Router,
};

use chrono::Utc;
use shared::http::{
    client::{SignInRequest, SignUpRequest},
    server::{GetUserResponse, SignInResponse, SignUpResponse},
};

use crate::{
    app_state::AppState,
    auth::{self, CurrentUser},
    constants, db,
};

use super::AppError;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/account/signup", post(post_sign_up))
        .route("/account/signin", post(post_sign_in))
        .route(
            "/account/me",
            get(get_me).layer(middleware::from_fn(auth::authorization_middleware)),
        )
}

async fn post_sign_up(
    State(db_pool): State<db::DbPool>,
    Json(payload): Json<SignUpRequest>,
) -> Result<Json<SignUpResponse>, AppError> {
    // TODO: middleware?
    auth::verify_captcha(&payload.captcha_token).await?;

    if !payload.accepted_terms {
        return Err(AppError::Forbidden);
    }

    match db::users::create_user(
        &db_pool,
        &payload.username,
        payload.email.as_deref().map(String::as_str),
        &auth::hash_password(&payload.password)?,
        &Utc::now(),
        constants::DEFAULT_MAX_CHARACTERS,
    )
    .await?
    {
        Some(_) => Ok(Json(SignUpResponse {})),
        None => Err(AppError::UserError("user already exists".to_string())),
    }
}

// TODO: move to auth api ?

async fn post_sign_in(
    State(db_pool): State<db::DbPool>,
    Json(payload): Json<SignInRequest>,
) -> Result<Json<SignInResponse>, AppError> {
    auth::verify_captcha(&payload.captcha_token).await?;

    Ok(Json(SignInResponse {
        jwt: auth::sign_in(
            &db_pool,
            &payload.username.into_inner(),
            &payload.password.into_inner(),
        )
        .await?,
    }))
}

async fn get_me(
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<GetUserResponse>, AppError> {
    Ok(Json(GetUserResponse {
        user: current_user.user,
    }))
}
