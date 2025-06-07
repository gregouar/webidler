use axum::extract::FromRef;

pub use crate::{
    db::pool::DbPool,
    game::{data::master_store::MasterStore, sessions::SessionsStore},
};

#[derive(Debug, Clone)]
pub struct AppState {
    pub db_pool: DbPool,
    pub master_store: MasterStore,
    pub sessions_store: SessionsStore,
}

impl FromRef<AppState> for DbPool {
    fn from_ref(app_state: &AppState) -> DbPool {
        app_state.db_pool.clone()
    }
}
impl FromRef<AppState> for MasterStore {
    fn from_ref(app_state: &AppState) -> MasterStore {
        app_state.master_store.clone()
    }
}
impl FromRef<AppState> for SessionsStore {
    fn from_ref(app_state: &AppState) -> SessionsStore {
        app_state.sessions_store.clone()
    }
}
