use anyhow::Result;

use axum::{
    extract::State,
    middleware,
    routing::{get, post},
    Extension, Json, Router,
};

use shared::{
    data::skill::SkillSpecs,
    http::{
        client::AscendPassivesRequest,
        server::{
            AscendPassivesResponse, GetAreasResponse, GetPassivesResponse, GetSkillsResponse,
        },
    },
};

use crate::{
    app_state::{AppState, MasterStore},
    auth::{self, CurrentUser},
    db,
    game::{data::DataInit, systems::passives_controller},
};

use super::AppError;

pub fn routes(app_state: AppState) -> Router<AppState> {
    let auth_routes = Router::new()
        .route("/game/passives", post(post_ascend_passives))
        .layer(middleware::from_fn_with_state(
            app_state,
            auth::authorization_middleware,
        ));

    Router::new()
        .route("/game/areas", get(get_areas))
        .route("/game/skills", get(get_skills))
        .route("/game/passives", get(get_passives))
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

    if character.user_id != current_user.user.user_id {
        return Err(AppError::Forbidden);
    }

    if character.area_id.is_some() {
        return Err(AppError::UserError("character is grinding".to_string()));
    }

    let (_, prev_ascension) =
        db::characters_data::load_character_data(&mut *tx, &payload.character_id)
            .await?
            .unwrap_or_default();

    let cost = passives_controller::validate_ascension(
        master_store
            .passives_store
            .get("default")
            .ok_or(anyhow::anyhow!("passives tree not found"))?,
        &payload.passives_tree_ascension,
    )? - passives_controller::compute_ascension_cost(&prev_ascension);

    if cost as f64 > character.resource_shards {
        return Err(AppError::UserError("not enough power shards".to_string()));
    }

    db::characters::update_character_resources(
        &mut *tx,
        &character.character_id,
        character.resource_gems,
        character.resource_shards - cost,
    )
    .await?;

    db::characters_data::save_character_passives(
        &mut *tx,
        &character.character_id,
        &payload.passives_tree_ascension,
    )
    .await?;

    tx.commit().await?;

    Ok(Json(AscendPassivesResponse {}))
}
