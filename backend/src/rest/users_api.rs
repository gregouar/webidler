use axum::{
    extract::State,
    middleware,
    routing::{get, post},
    Extension, Json, Router,
};

use chrono::Utc;
use shared::{
    constants::DEFAULT_MAX_CHARACTERS,
    http::{
        client::{ForgotPasswordRequest, SignInRequest, SignUpRequest},
        server::{ForgotPasswordResponse, GetUserDetailsResponse, SignInResponse, SignUpResponse},
    },
};

use crate::{
    app_state::{AppSettings, AppState},
    auth::{self, CurrentUser},
    db,
    email::EmailService,
};

use super::AppError;

pub fn routes(app_state: AppState) -> Router<AppState> {
    let auth_routes =
        Router::new()
            .route("/account/me", get(get_me))
            .layer(middleware::from_fn_with_state(
                app_state,
                auth::authorization_middleware,
            ));

    Router::new()
        .route("/account/signup", post(post_sign_up))
        .route("/account/signin", post(post_sign_in))
        .route("/account/forgot-password", post(post_forgot_password))
        .merge(auth_routes)
}

async fn post_sign_up(
    State(app_settings): State<AppSettings>,
    State(db_pool): State<db::DbPool>,
    Json(payload): Json<SignUpRequest>,
) -> Result<Json<SignUpResponse>, AppError> {
    // TODO: middleware?
    auth::verify_captcha(&payload.captcha_token).await?;

    if !payload.accepted_terms {
        return Err(AppError::Forbidden);
    }

    let (email_crypt, email_hash) = match payload.email.as_deref() {
        Some(email) => {
            let crypt = Some(auth::encrypt_email(&app_settings, email)?);
            let hash = Some(auth::hash_email(&app_settings, email));
            (crypt, hash)
        }
        None => (None, None),
    };

    match db::users::create_user(
        &db_pool,
        &payload.username,
        email_crypt.as_deref(),
        email_hash.as_deref(),
        &auth::hash_password(&payload.password)?,
        &Utc::now(),
        DEFAULT_MAX_CHARACTERS as i16,
    )
    .await?
    {
        Some(_) => Ok(Json(SignUpResponse {})),
        None => Err(AppError::UserError("user already exists".to_string())),
    }
}

// TODO: move to auth api ?

async fn post_sign_in(
    State(app_settings): State<AppSettings>,
    State(db_pool): State<db::DbPool>,
    Json(payload): Json<SignInRequest>,
) -> Result<Json<SignInResponse>, AppError> {
    auth::verify_captcha(&payload.captcha_token).await?;

    Ok(Json(SignInResponse {
        jwt: auth::sign_in(
            &app_settings,
            &db_pool,
            &payload.username.into_inner(),
            &payload.password.into_inner(),
        )
        .await?,
    }))
}

async fn get_me(
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<GetUserDetailsResponse>, AppError> {
    Ok(Json(GetUserDetailsResponse {
        user_details: current_user.user_details,
    }))
}

async fn post_forgot_password(
    State(app_settings): State<AppSettings>,
    State(email_service): State<EmailService>,
    State(db_pool): State<db::DbPool>,
    Json(payload): Json<ForgotPasswordRequest>,
) -> Result<Json<ForgotPasswordResponse>, AppError> {
    auth::verify_captcha(&payload.captcha_token).await?;

    let user = db::users::read_user_by_email(
        &db_pool,
        &auth::hash_email(&app_settings, payload.email.as_str()),
    )
    .await?
    .ok_or_else(|| AppError::NotFound)?;

    // TODO add entry to password reset table

    email_service
        .send_email(payload.email, "Hello", "hello there".into())
        .await?;

    Ok(Json(ForgotPasswordResponse {}))
}
