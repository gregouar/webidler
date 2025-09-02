use axum::{
    extract::{Path, State},
    middleware,
    routing::{delete, get, post},
    Extension, Json, Router,
};
use shared::{
    data::{
        area::AreaLevel,
        player::{EquippedSlot, PlayerInventory},
        user::{UserCharacter, UserCharacterActivity, UserCharacterId, UserGrindArea, UserId},
    },
    http::{
        client::CreateCharacterRequest,
        server::{
            CreateCharacterResponse, DeleteCharacterResponse, GetCharacterDetailsResponse,
            GetUserCharactersResponse,
        },
    },
};

use crate::{
    app_state::{AppState, MasterStore},
    auth::{self, CurrentUser},
    db,
    game::systems::items_controller,
};

use super::AppError;

pub fn routes(app_state: AppState) -> Router<AppState> {
    let auth_routes = Router::new()
        .route("/users/{user_id}/characters", post(post_create_character))
        .route("/characters/{character_id}", delete(delete_character))
        .layer(middleware::from_fn_with_state(
            app_state,
            auth::authorization_middleware,
        ));

    Router::new()
        .route("/users/{user_id}/characters", get(get_user_characters))
        .route("/characters/{character_id}", get(get_character_details))
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

async fn get_character_details(
    State(db_pool): State<db::DbPool>,
    State(master_store): State<MasterStore>,
    Path(character_id): Path<UserId>,
) -> Result<Json<GetCharacterDetailsResponse>, AppError> {
    let (character, areas_completed, character_data) = tokio::join!(
        db::characters::read_character(&db_pool, &character_id),
        db::characters::read_character_areas_completed(&db_pool, &character_id),
        db::characters_data::load_character_data(&db_pool, &character_id)
    );

    let character = character?.ok_or(AppError::NotFound)?.into();
    let areas_completed = areas_completed?;
    let (inventory_data, ascension) = character_data?.unwrap_or_default();

    let areas = master_store
        .area_blueprints_store
        .iter()
        .map(|(area_id, available_area)| UserGrindArea {
            area_id: area_id.clone(),
            area_specs: available_area.specs.clone(),
            max_level_reached: areas_completed
                .iter()
                .find(|area_completed| area_completed.area_id.eq(area_id))
                .map(|area_completed| area_completed.max_area_level as AreaLevel)
                .unwrap_or_default(),
        })
        .collect();

    let inventory = PlayerInventory {
        equipped: inventory_data
            .equipped
            .into_iter()
            .filter_map(|(item_slot, item_modifiers)| {
                Some((
                    item_slot,
                    EquippedSlot::MainSlot(Box::new(items_controller::init_item_specs_from_store(
                        &master_store.items_store,
                        item_modifiers,
                    )?)),
                ))
            })
            .collect(),
        bag: inventory_data
            .bag
            .into_iter()
            .filter_map(|item_modifiers| {
                items_controller::init_item_specs_from_store(
                    &master_store.items_store,
                    item_modifiers,
                )
            })
            .collect(),
        max_bag_size: inventory_data.max_bag_size,
    };

    Ok(Json(GetCharacterDetailsResponse {
        character,
        areas,
        inventory,
        ascension,
    }))
}

async fn delete_character(
    State(db_pool): State<db::DbPool>,
    Path(character_id): Path<UserCharacterId>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<DeleteCharacterResponse>, AppError> {
    let character = db::characters::read_character(&db_pool, &character_id).await?;

    if !character
        .map(|character| character.user_id == current_user.user.user_id)
        .unwrap_or_default()
    {
        return Err(AppError::NotFound);
    }

    db::characters::delete_character(&db_pool, &character_id).await?;
    Ok(Json(DeleteCharacterResponse {}))
}

impl From<db::characters::CharacterEntry> for UserCharacter {
    fn from(val: db::characters::CharacterEntry) -> Self {
        UserCharacter {
            character_id: val.character_id,
            name: val.character_name,
            portrait: val.portrait,
            resource_gems: val.resource_gems,
            resource_shards: val.resource_shards,
            max_area_level: val.max_area_level as AreaLevel,
            activity: if let (Some(area_id), Some(area_level)) = (val.area_id, val.area_level) {
                UserCharacterActivity::Grinding(area_id, area_level as AreaLevel)
            } else {
                UserCharacterActivity::Rusting
            },
        }
    }
}
