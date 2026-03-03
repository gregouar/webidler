use axum::Router;

use crate::app_state::AppState;

mod app_error;
mod messages;
mod moderation;

pub use app_error::AppError;

pub fn routes() -> Router<AppState> {
    Router::new()
        .merge(messages::routes())
        .merge(moderation::routes())
}
