use axum::Router;

use crate::app_state::AppState;

mod app_error;
mod characters_api;
mod forge_api;
mod game_api;
mod inventory_api;
mod leaderboard_api;
mod market_api;
mod stats_api;
mod users_api;
pub mod utils;

pub use app_error::AppError;

pub fn routes(app_state: AppState) -> Router<AppState> {
    Router::new()
        .merge(characters_api::routes(app_state.clone()))
        .merge(game_api::routes(app_state.clone()))
        .merge(leaderboard_api::routes())
        .merge(stats_api::routes())
        .merge(users_api::routes(app_state.clone()))
        .merge(market_api::routes(app_state.clone()))
        .merge(forge_api::routes(app_state.clone()))
        .merge(inventory_api::routes(app_state.clone()))
}
