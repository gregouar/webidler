use anyhow::Result;

use axum::{extract::State, routing::get, Json, Router};

use shared::{
    data::skill::SkillSpecs,
    http::server::{GetAreasResponse, GetPassivesResponse, GetSkillsResponse},
};

use crate::{
    app_state::{AppState, MasterStore},
    game::data::DataInit,
};

use super::AppError;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/game/areas", get(get_areas))
        .route("/game/skills", get(get_skills))
        .route("/game/passives", get(get_passives))
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
