use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Instant,
};

use shared::messages::SessionKey;

use super::game_data::GameInstanceData;

#[derive(Debug, Clone)]
pub struct SessionsStore {
    pub sessions: Arc<Mutex<HashMap<String, Session>>>,
    pub players: Arc<Mutex<usize>>,
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
            players: Arc::new(Mutex::new(0)),
        }
    }
}
