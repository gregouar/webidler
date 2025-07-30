use anyhow::Result;

use axum::{extract::State, routing::get, Json, Router};

use shared::{data::skill::SkillSpecs, http::server::SkillsResponse};

use crate::{
    app_state::{AppState, MasterStore},
    game::data::DataInit,
};

use super::AppError;

pub fn routes() -> Router<AppState> {
    Router::new().route("/game/skills", get(get_skills))
}

pub async fn get_skills(
    State(master_store): State<MasterStore>,
) -> Result<Json<SkillsResponse>, AppError> {
    Ok(Json(SkillsResponse {
        skills: master_store
            .skills_store
            .iter()
            .map(|(k, v)| (k.clone(), SkillSpecs::init(v.clone())))
            .collect(),
    }))
}
