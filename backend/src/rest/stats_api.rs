use axum::{extract::State, routing::get, Json, Router};

use shared::http::server::PlayersCountResponse;

use crate::{app_state::AppState, db};

use super::AppError;

pub fn routes() -> Router<AppState> {
    Router::new().route("/players", get(get_players_count))
}

async fn get_players_count(
    State(db_pool): State<db::DbPool>,
) -> Result<Json<PlayersCountResponse>, AppError> {
    Ok(Json(PlayersCountResponse {
        value: db::game_sessions::count_active_sessions(&db_pool).await?,
    }))
}
