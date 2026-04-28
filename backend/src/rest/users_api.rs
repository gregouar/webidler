use std::sync::Arc;

use axum::{
    Extension, Json, Router,
    extract::{Path, State},
    middleware,
    routing::{delete, get, post},
};

use backend_shared::profanities_checker::ProfanitiesChecker;
use chrono::{Duration, Utc};

use shared::{
    constants::DEFAULT_MAX_CHARACTERS,
    data::user::{UserDetails, UserId},
    http::{
        client::{
            ForgotPasswordRequest, ResetPasswordRequest, SignInRequest, SignUpRequest,
            UpdateAccountRequest,
        },
        server::{
            DeleteAccountResponse, ForgotPasswordResponse, GetDiscordInviteResponse,
            GetUserDetailsResponse, ResetPasswordResponse, SignInResponse, SignUpResponse,
            UpdateAccountResponse,
        },
    },
};

use crate::{
    app_state::{AppSettings, AppState},
    auth::{self, User},
    db::{self, users::UserUpdate},
    email::EmailService,
    integration::discord::DiscordIntegration,
};

use super::AppError;

pub fn routes(app_state: AppState) -> Router<AppState> {
    let auth_routes = Router::new()
        .route("/account/me", get(get_me))
        .route("/account/update", post(post_update_account))
        .route("/account/{user_id}", delete(delete_account))
        .route("/discord", get(get_discord_invite))
        .layer(middleware::from_fn_with_state(
            app_state,
            auth::authorization_middleware,
        ));

    Router::new()
        .route("/account/signup", post(post_sign_up))
        .route("/account/signin", post(post_sign_in))
        .route("/account/forgot-password", post(post_forgot_password))
        .route("/account/reset-password", post(post_reset_password))
        .merge(auth_routes)
}

async fn post_sign_up(
    State(app_settings): State<AppSettings>,
    State(db_pool): State<db::DbPool>,
    State(profanities_checker): State<Arc<ProfanitiesChecker>>,
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
            let hash = Some(auth::hash_content(&app_settings, email));
            (crypt, hash)
        }
        None => (None, None),
    };

    if profanities_checker
        .find_profanity(&payload.username)
        .is_some()
    {
        return Err(AppError::UserError(
            "this name contains inappropriate language, please choose a different name".into(),
        ));
    }

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

async fn get_discord_invite(
    Extension(user): Extension<User>,
    State(discord): State<DiscordIntegration>,
) -> Result<Json<GetDiscordInviteResponse>, AppError> {
    let code = discord.get_invite(user.user_id).await?;

    Ok(Json(GetDiscordInviteResponse { code }))
}

async fn get_me(
    State(app_settings): State<AppSettings>,
    State(db_pool): State<db::DbPool>,
    Extension(user): Extension<User>,
) -> Result<Json<GetUserDetailsResponse>, AppError> {
    let user = db::users::read_user(&db_pool, &user.user_id)
        .await?
        .ok_or_else(|| AppError::Unauthorized("invalid token".to_string()))?;

    let email = user
        .email_crypt
        .as_ref()
        .and_then(|email_crypt| auth::decrypt_email(&app_settings, email_crypt).ok());

    Ok(Json(GetUserDetailsResponse {
        user_details: UserDetails {
            max_characters: user.max_characters as u8,
            chat_badge: user.chat_badge.clone(),
            user: user.into(),
            email,
        },
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
        &auth::hash_content(&app_settings, payload.email.as_str()),
    )
    .await?
    .ok_or_else(|| AppError::NotFound)?;

    let token = auth::generate_token();
    let expires_at = Utc::now() + Duration::minutes(30);
    let token_hash = auth::hash_content(&app_settings, &token);
    db::password_reset::create_password_reset(
        &db_pool,
        &user.user_id,
        &token_hash,
        expires_at.into(),
    )
    .await?;

    let reset_link = format!(
        "{}/reset-password?user_id={}&token={}",
        app_settings.frontend_url, &user.user_id, token,
    );

    let subject = "Reset your password";
    let html_content = format!(
        "<p>Hello {},</p>
         <p>Click <a href=\"{reset_link}\">here</a> to reset your password on Grind to Rust.</p>
         <p>If you didn't request this, you can safely ignore this email.</p>",
        user.username.unwrap_or_default()
    );
    let text_content =
        format!("Visit this link to reset your password on Grind to Rust: {reset_link}");

    email_service
        .send_email(payload.email, subject, html_content, text_content)
        .await?;

    Ok(Json(ForgotPasswordResponse {}))
}

async fn post_reset_password(
    State(app_settings): State<AppSettings>,
    State(db_pool): State<db::DbPool>,
    Json(payload): Json<ResetPasswordRequest>,
) -> Result<Json<ResetPasswordResponse>, AppError> {
    auth::verify_captcha(&payload.captcha_token).await?;

    let token_hash = auth::hash_content(&app_settings, &payload.password_token);

    let mut tx = db_pool.begin().await?;

    db::password_reset::redeem_password_reset(&mut tx, &payload.user_id, &token_hash)
        .await?
        .ok_or_else(|| AppError::UserError("invalid token".into()))?;
    db::users::update_user(
        &mut tx,
        &payload.user_id,
        &UserUpdate {
            password_hash: Some(auth::hash_password(&payload.password)?),
            ..Default::default()
        },
    )
    .await?;

    tx.commit().await?;

    Ok(Json(ResetPasswordResponse {}))
}

async fn post_update_account(
    Extension(user): Extension<User>,
    State(app_settings): State<AppSettings>,
    State(db_pool): State<db::DbPool>,
    State(profanities_checker): State<Arc<ProfanitiesChecker>>,
    Json(payload): Json<UpdateAccountRequest>,
) -> Result<Json<UpdateAccountResponse>, AppError> {
    let (email_crypt, email_hash) = if let Some(email) = payload.email {
        match email.as_deref() {
            Some(email) => {
                let crypt = Some(Some(auth::encrypt_email(&app_settings, email)?));
                let hash = Some(Some(auth::hash_content(&app_settings, email)));
                (crypt, hash)
            }
            None => (Some(None), Some(None)),
        }
    } else {
        (None, None)
    };

    // Double check authentication when trying to reset password
    if payload.password.is_some() {
        auth::sign_in(
            &app_settings,
            &db_pool,
            &user.username,
            &payload
                .old_password
                .map(|p| p.into_inner())
                .unwrap_or_default(),
        )
        .await?;
    }

    let user_update = UserUpdate {
        username: payload.username.map(|u| u.into_inner()),
        email_crypt,
        email_hash,
        password_hash: payload
            .password
            .and_then(|password| auth::hash_password(&password).ok()),
    };

    if let Some(username) = user_update.username.as_ref()
        && profanities_checker.find_profanity(username).is_some()
    {
        return Err(AppError::UserError(
            "this name contains inappropriate language, please choose a different name".into(),
        ));
    }
    let mut tx = db_pool.begin().await?;

    let r = match db::users::update_user(&mut tx, &user.user_id, &user_update).await? {
        Some(_) => Ok(Json(UpdateAccountResponse {})),
        None => Err(AppError::UserError(
            "username or email already in use".to_string(),
        )),
    };

    tx.commit().await?;

    r
}

async fn delete_account(
    State(db_pool): State<db::DbPool>,
    Path(user_id): Path<UserId>,
    Extension(user): Extension<User>,
) -> Result<Json<DeleteAccountResponse>, AppError> {
    if user.user_id != user_id {
        return Err(AppError::Forbidden);
    }

    db::users::delete_user(&db_pool, &user_id).await?;
    Ok(Json(DeleteAccountResponse {}))
}
