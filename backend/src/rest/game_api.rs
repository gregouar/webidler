use anyhow::Result;

use axum::{
    extract::State,
    middleware,
    routing::{get, post},
    Extension, Json, Router,
};

use shared::{
    data::{area::AreaLevel, skill::SkillSpecs},
    http::{
        client::{AscendPassivesRequest, BuyBenedictionsRequest, SocketPassiveRequest},
        server::{
            AscendPassivesResponse, BuyBenedictionsResponse, GetAreasResponse,
            GetBenedictionsResponse, GetPassivesResponse, GetSkillsResponse, SocketPassiveResponse,
        },
    },
};

use crate::{
    app_state::{AppState, MasterStore},
    auth::{self, CurrentUser},
    db,
    game::{
        data::{
            inventory_data::inventory_data_to_player_inventory,
            passives::ascension_data_to_passives_tree_ascension, DataInit,
        },
        systems::{benedictions_controller, inventory_controller, passives_controller},
    },
    rest::utils::{verify_character_in_town, verify_character_user},
};

use super::AppError;

pub fn routes(app_state: AppState) -> Router<AppState> {
    let auth_routes = Router::new()
        .route("/game/passives", post(post_ascend_passives))
        .route("/game/passives/socket", post(post_socket_passive))
        .route("/game/benedictions", post(post_buy_benedictions))
        .layer(middleware::from_fn_with_state(
            app_state,
            auth::authorization_middleware,
        ));

    Router::new()
        .route("/game/areas", get(get_areas))
        .route("/game/skills", get(get_skills))
        .route("/game/passives", get(get_passives))
        .route("/game/benedictions", get(get_benedictions))
        .merge(auth_routes)
}

pub async fn get_areas(
    State(master_store): State<MasterStore>,
) -> Result<Json<GetAreasResponse>, AppError> {
    Ok(Json(GetAreasResponse {
        areas: master_store
            .area_blueprints_store
            .iter()
            .map(|(k, v)| (k.clone(), v.specs.clone()))
            .collect(),
    }))
}

pub async fn get_skills(
    State(master_store): State<MasterStore>,
) -> Result<Json<GetSkillsResponse>, AppError> {
    Ok(Json(GetSkillsResponse {
        skills: master_store
            .skills_store
            .iter()
            .map(|(k, v)| (k.clone(), SkillSpecs::init(v.clone())))
            .collect(),
    }))
}

pub async fn get_passives(
    State(master_store): State<MasterStore>,
) -> Result<Json<GetPassivesResponse>, AppError> {
    Ok(Json(GetPassivesResponse {
        passives_tree_specs: master_store
            .passives_store
            .get("default")
            .cloned()
            .unwrap_or_default(),
    }))
}

pub async fn get_benedictions(
    State(master_store): State<MasterStore>,
) -> Result<Json<GetBenedictionsResponse>, AppError> {
    Ok(Json(GetBenedictionsResponse {
        benedictions_specs: master_store.benedictions_store.as_ref().clone(),
    }))
}

pub async fn post_ascend_passives(
    State(master_store): State<MasterStore>,
    State(db_pool): State<db::DbPool>,
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<AscendPassivesRequest>,
) -> Result<Json<AscendPassivesResponse>, AppError> {
    let mut tx = db_pool.begin().await?;

    let character = db::characters::read_character(&mut *tx, &payload.character_id)
        .await?
        .ok_or(AppError::NotFound)?;

    verify_character_user(&character, &current_user)?;
    verify_character_in_town(&character)?;

    let (_, ascension_data, _) =
        db::characters_data::load_character_data(&db_pool, &payload.character_id)
            .await?
            .ok_or(AppError::NotFound)?;

    let mut ascension =
        ascension_data_to_passives_tree_ascension(&master_store.items_store, ascension_data);

    ascension.ascended_nodes = payload.ascended_nodes;

    passives_controller::update_ascension(
        &mut tx,
        &master_store,
        &payload.character_id,
        character.resource_shards,
        &ascension,
    )
    .await?;

    tx.commit().await?;

    let (character, character_data) = tokio::join!(
        db::characters::read_character(&db_pool, &payload.character_id),
        db::characters_data::load_character_data(&db_pool, &payload.character_id)
    );

    let character = character?.ok_or(AppError::NotFound)?.into();
    let (_, ascension_data, _) = character_data?.unwrap_or_default();

    let ascension =
        ascension_data_to_passives_tree_ascension(&master_store.items_store, ascension_data);

    Ok(Json(AscendPassivesResponse {
        character,
        ascension,
    }))
}

pub async fn post_socket_passive(
    State(master_store): State<MasterStore>,
    State(db_pool): State<db::DbPool>,
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<SocketPassiveRequest>,
) -> Result<Json<SocketPassiveResponse>, AppError> {
    let mut tx = db_pool.begin().await?;

    let character = db::characters::read_character(&mut *tx, &payload.character_id)
        .await?
        .ok_or(AppError::NotFound)?;

    verify_character_user(&character, &current_user)?;
    verify_character_in_town(&character)?;

    let (inventory_data, ascension_data, _) =
        db::characters_data::load_character_data(&db_pool, &payload.character_id)
            .await?
            .ok_or(AppError::NotFound)?;

    let mut inventory =
        inventory_data_to_player_inventory(&master_store.items_store, inventory_data);
    let mut ascension =
        ascension_data_to_passives_tree_ascension(&master_store.items_store, ascension_data);

    let item_specs = payload.item_index.and_then(|item_index| {
        inventory_controller::remove_item_from_bag(&mut inventory, item_index).ok()
    });

    let removed_item = passives_controller::socket_node(
        &master_store,
        character.max_area_level as AreaLevel,
        &mut ascension,
        payload.passive_node_id,
        item_specs,
    )?;

    if let Some(item_specs) = removed_item {
        inventory_controller::store_item_to_bag(&mut inventory, item_specs)?;
    }

    db::characters_data::save_character_passives(&mut *tx, &payload.character_id, &ascension)
        .await?;
    db::characters_data::save_character_inventory(&mut *tx, &payload.character_id, &inventory)
        .await?;

    tx.commit().await?;

    Ok(Json(SocketPassiveResponse {
        ascension,
        inventory,
    }))
}

pub async fn post_buy_benedictions(
    State(master_store): State<MasterStore>,
    State(db_pool): State<db::DbPool>,
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<BuyBenedictionsRequest>,
) -> Result<Json<BuyBenedictionsResponse>, AppError> {
    let mut tx = db_pool.begin().await?;

    let character = db::characters::read_character(&mut *tx, &payload.character_id)
        .await?
        .ok_or(AppError::NotFound)?;

    verify_character_user(&character, &current_user)?;
    verify_character_in_town(&character)?;

    benedictions_controller::update_benedictions(
        &mut tx,
        &master_store,
        &payload.character_id,
        character.resource_gold,
        &payload.player_benedictions,
    )
    .await?;

    tx.commit().await?;

    let (character, character_data) = tokio::join!(
        db::characters::read_character(&db_pool, &payload.character_id),
        db::characters_data::load_character_data(&db_pool, &payload.character_id)
    );

    let character = character?.ok_or(AppError::NotFound)?.into();
    let (_, _, player_benedictions) = character_data?.unwrap_or_default();

    Ok(Json(BuyBenedictionsResponse {
        character,
        player_benedictions,
    }))
}
