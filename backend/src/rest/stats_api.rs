use axum::{extract::State, routing::get, Json, Router};

use shared::http::server::{LeaderboardEntry, PlayersCountResponse};

use crate::{app_state::AppState, db};

use super::AppError;

pub fn routes() -> Router<AppState> {
    Router::new().route("/players", get(get_players_count))
}

async fn get_players_count(
    State(db_pool): State<db::DbPool>,
) -> Result<Json<PlayersCountResponse>, AppError> {
    let value = db::game_sessions::count_active_sessions(&db_pool).await?;
    let glimpse = db::game_sessions::glimpse_active_sessions(&db_pool, 10)
        .await?
        .into_iter()
        .map(|entry| LeaderboardEntry {
            user_id: entry.user_id,
            username: entry.username.unwrap_or("Hidden User".into()),
            character_id: entry.character_id,
            character_name: entry.character_name,
            area_id: entry.area_id,
            area_level: entry.area_level as u16,
            created_at: entry.created_at.into(),
            elapsed_time: None,
            comments: "".into(),
        })
        .collect();
    Ok(Json(PlayersCountResponse { value, glimpse }))
}
