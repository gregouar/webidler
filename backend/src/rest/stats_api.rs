use axum::{extract::State, routing::get, Json, Router};

use shared::http::server::PlayersCountResponse;

use crate::app_state::{AppState, SessionsStore};

pub fn routes() -> Router<AppState> {
    Router::new().route("/players", get(get_players_count))
}

async fn get_players_count(
    State(sessions_store): State<SessionsStore>,
) -> Json<PlayersCountResponse> {
    Json(PlayersCountResponse {
        value: *sessions_store.players.lock().unwrap(),
    })
}
