use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Instant,
};

use shared::messages::SessionKey;

use super::game_instance_data::GameInstanceData;

#[derive(Debug, Clone)]
pub struct SessionsStore {
    pub sessions: Arc<Mutex<HashMap<String, Session>>>,
}

#[derive(Debug, Clone)]
pub struct Session {
    pub session_key: SessionKey,
    pub last_active: Instant,

    pub data: Box<GameInstanceData>,
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
