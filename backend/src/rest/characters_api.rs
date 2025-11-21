use axum::{
    extract::{Path, State},
    middleware,
    routing::{delete, get, post},
    Extension, Json, Router,
};
use shared::{
    data::{
        area::AreaLevel,
        game_stats::GrindStats,
        player::{EquippedSlot, PlayerInventory},
        skill::SkillSpecs,
        user::{UserCharacter, UserCharacterActivity, UserCharacterId, UserGrindArea, UserId},
    },
    http::{
        client::CreateCharacterRequest,
        server::{
            CreateCharacterResponse, DeleteCharacterResponse, GetCharacterDetailsResponse,
            GetUserCharactersResponse,
        },
    },
    types::Username,
};

use crate::{
    app_state::{AppState, MasterStore},
    auth::{self, CurrentUser},
    db,
    game::{data::DataInit, systems::items_controller},
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
        .route(
            "/view-character/{character_name}",
            get(get_character_by_name),
        )
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
    if current_user.user_details.user.user_id != user_id {
        return Err(AppError::Forbidden);
    }

    match db::characters::create_character(
        &db_pool,
        &user_id,
        &payload.name,
        &format!("adventurers/{}.webp", payload.portrait.into_inner()),
    )
    .await?
    {
        Some(character_id) => Ok(Json(CreateCharacterResponse { character_id })),
        None => Err(AppError::UserError("name already taken".to_string())),
    }
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
    read_character_details(db_pool, master_store, character_id).await
}

async fn get_character_by_name(
    State(db_pool): State<db::DbPool>,
    State(master_store): State<MasterStore>,
    Path(character_name): Path<Username>,
) -> Result<Json<GetCharacterDetailsResponse>, AppError> {
    let character_id = db::characters::get_character_by_name(&db_pool, &character_name)
        .await?
        .ok_or(AppError::UserError(format!(
            "character '{}' not found",
            character_name.into_inner()
        )))?;

    read_character_details(db_pool, master_store, character_id).await
}

async fn read_character_details(
    db_pool: db::DbPool,
    master_store: MasterStore,
    character_id: UserCharacterId,
) -> Result<Json<GetCharacterDetailsResponse>, AppError> {
    let (character, areas_completed, character_data, last_grind_data) = tokio::join!(
        db::characters::read_character(&db_pool, &character_id),
        db::characters::read_character_areas_completed(&db_pool, &character_id),
        db::characters_data::load_character_data(&db_pool, &character_id),
        db::game_stats::load_last_game_stats(&db_pool, &character_id)
    );

    let character = character?.ok_or(AppError::NotFound)?.into();
    let areas_completed = areas_completed?;
    let (inventory_data, ascension, benedictions) = character_data?.unwrap_or_default();
    let last_grind_data = last_grind_data?;

    let areas = master_store
        .area_blueprints_store
        .iter()
        .map(|(area_id, available_area)| UserGrindArea {
            area_id: area_id.clone(),
            area_specs: available_area.specs.clone(),
            max_level_reached: areas_completed
                .iter()
                .find(|area_completed| area_completed.area_id.eq(area_id))
                .map(|area_completed| {
                    area_completed.max_area_level as AreaLevel + available_area.specs.starting_level
                        - 1
                })
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

    let last_grind = last_grind_data.map(|last_grind_data| {
        let (items_data, skills_data) = last_grind_data;

        let mut skills_specs: Vec<_> = items_data
            .map(|items_data| {
                items_data
                    .values()
                    .flat_map(|equipped_slot| match equipped_slot {
                        EquippedSlot::MainSlot(item_specs) => {
                            item_specs.weapon_specs.clone().map(|weapon_specs| {
                                SkillSpecs::init(items_controller::make_weapon_skill(
                                    item_specs.modifiers.level,
                                    &weapon_specs,
                                ))
                            })
                        }
                        _ => None,
                    })
                    .collect()
            })
            .unwrap_or_default();

        skills_specs.extend(
            skills_data
                .unwrap_or_default()
                .into_iter()
                .flat_map(|skill_id| master_store.skills_store.get(&skill_id))
                .map(|base_skill_specs| SkillSpecs::init(base_skill_specs.clone())),
        );

        GrindStats { skills_specs }
    });

    Ok(Json(GetCharacterDetailsResponse {
        character,
        areas,
        inventory,
        ascension,
        benedictions,
        last_grind,
    }))
}

async fn delete_character(
    State(db_pool): State<db::DbPool>,
    Path(character_id): Path<UserCharacterId>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<DeleteCharacterResponse>, AppError> {
    let character = db::characters::read_character(&db_pool, &character_id).await?;

    if !character
        .map(|character| character.user_id == current_user.user_details.user.user_id)
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
            resource_gold: val.resource_gold,
            max_area_level: val.max_area_level as AreaLevel,
            activity: if let (Some(area_id), Some(area_level)) = (val.area_id, val.area_level) {
                UserCharacterActivity::Grinding(area_id, area_level as AreaLevel)
            } else {
                UserCharacterActivity::Rusting
            },
        }
    }
}
