use dashmap::{DashMap, DashSet};
use std::{sync::Arc, time::Instant};

pub use shared::data::user::UserCharacterId;

use super::game_data::GameInstanceData;

#[derive(Debug, Clone)]
pub struct SessionsStore {
    pub sessions: Arc<DashMap<UserCharacterId, Session>>,
    pub sessions_stealing: Arc<DashSet<UserCharacterId>>,
    // pub sessions: Arc<Mutex<HashMap<UserCharacterId, Session>>>,
    // pub sessions_stealing: Arc<Mutex<HashSet<UserCharacterId>>>,
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
            sessions: Default::default(),
            sessions_stealing: Default::default(),
        }
    }
}
