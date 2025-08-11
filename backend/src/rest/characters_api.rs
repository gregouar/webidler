use axum::{
    extract::{Path, State},
    middleware,
    routing::{get, post},
    Extension, Json, Router,
};
use shared::{
    data::user::{self, UserCharacter, UserId},
    http::{
        client::CreateCharacterRequest,
        server::{CreateCharacterResponse, GetUserCharactersResponse},
    },
};

use crate::{
    app_state::AppState,
    auth::{self, CurrentUser},
    db,
};

use super::AppError;

pub fn routes(app_state: AppState) -> Router<AppState> {
    let auth_routes = Router::new()
        .route("/users/{user_id}/characters", post(post_create_character))
        .layer(middleware::from_fn_with_state(
            app_state,
            auth::authorization_middleware,
        ));

    Router::new()
        .route("/users/{user_id}/characters", get(get_user_characters))
        .merge(auth_routes)
    // .route("characters/{character_id}", get(get_character))
}

async fn post_create_character(
    State(db_pool): State<db::DbPool>,
    Path(user_id): Path<UserId>,
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<CreateCharacterRequest>,
) -> Result<Json<CreateCharacterResponse>, AppError> {
    // TODO: better access management
    if current_user.user.user_id != user_id {
        return Err(AppError::Forbidden);
    }

    Ok(Json(CreateCharacterResponse {
        character_id: db::characters::create_character(
            &db_pool,
            &user_id,
            &payload.name,
            &format!("adventurers/{}.webp", payload.portrait.into_inner()),
        )
        .await?,
    }))
}

async fn get_user_characters(
    State(db_pool): State<db::DbPool>,
    Path(user_id): Path<UserId>,
) -> Result<Json<GetUserCharactersResponse>, AppError> {
    Ok(Json(GetUserCharactersResponse {
        characters: db::characters::read_all_user_characters(&db_pool, &user_id)
            .await?
            .into_iter()
            .map(|c| c.into())
            .collect(),
    }))
}

impl Into<UserCharacter> for db::characters::CharacterEntry {
    fn into(self) -> UserCharacter {
        UserCharacter {
            character_id: self.character_id,
            name: self.character_name,
            portrait: self.portrait,
            max_area_level: self.max_area_level,
            activity: user::UserCharacterActivity::Idle, // TODO
        }
    }
}
