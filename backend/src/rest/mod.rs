use axum::Router;

use crate::app_state::AppState;

mod app_error;
mod game_api;
mod leaderboard_api;
mod stats_api;

pub use app_error::AppError;

pub fn routes() -> Router<AppState> {
    Router::new()
        .merge(game_api::routes())
        .merge(leaderboard_api::routes())
        .merge(stats_api::routes())
}
