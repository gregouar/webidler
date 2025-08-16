use axum::Router;

use crate::app_state::AppState;

mod app_error;
mod characters_api;
mod game_api;
mod leaderboard_api;
mod stats_api;
mod users_api;

pub use app_error::AppError;

pub fn routes(app_state: AppState) -> Router<AppState> {
    Router::new()
        .merge(characters_api::routes(app_state.clone()))
        .merge(game_api::routes())
        .merge(leaderboard_api::routes())
        .merge(stats_api::routes())
        .merge(users_api::routes(app_state.clone()))
}
