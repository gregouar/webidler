use axum::Router;

use crate::app_state::AppState;

mod app_error;
mod leaderboard_api;
mod stats_api;

pub use app_error::AppError;

pub fn routes() -> Router<AppState> {
    Router::new()
        .merge(leaderboard_api::routes())
        .merge(stats_api::routes())
}
