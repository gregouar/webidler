use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
    time::Instant,
};

pub use shared::data::user::UserCharacterId;

use super::game_data::GameInstanceData;

#[derive(Debug, Clone)]
pub struct SessionsStore {
    pub sessions: Arc<Mutex<HashMap<UserCharacterId, Session>>>,
    pub sessions_stealing: Arc<Mutex<HashSet<UserCharacterId>>>,
}

#[derive(Debug, Clone)]
pub struct Session {
    pub character_id: UserCharacterId,
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
            sessions_stealing: Default::default(),
        }
    }
}
