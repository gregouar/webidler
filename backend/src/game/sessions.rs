use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Instant,
};

pub use shared::messages::{SessionId, SessionKey, UserId};

use super::game_data::GameInstanceData;

#[derive(Debug, Clone)]
pub struct SessionsStore {
    pub sessions: Arc<Mutex<HashMap<SessionId, Session>>>,
}

#[derive(Debug, Clone)]
pub struct Session {
    pub user_id: UserId,
    pub session_key: SessionKey,
    pub last_active: Instant,

    pub game_data: Box<GameInstanceData>,
}

impl Default for SessionsStore {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionsStore {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}
