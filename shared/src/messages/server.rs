use serde::{Deserialize, Serialize};

use crate::data::{MonsterPrototype, MonsterState, PlayerPrototype, PlayerState};

use super::macros::impl_into_message;

impl_into_message! {
    #[derive(Serialize, Deserialize, Debug, Clone,)]
    pub enum ServerMessage {
        Connect(ConnectMessage),
        InitGame(InitGameMessage),
        UpdateGame(SyncGameStateMessage),
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConnectMessage {
    pub greeting: String,
    pub value: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InitGameMessage {
    pub player_prototype: PlayerPrototype,
    pub player_state: PlayerState,
}

/// Message to be sent every tick to sync current state of the game with clients
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SyncGameStateMessage {
    pub value: i32,
    pub player_state: PlayerState,
    // Monsters list is only updated when monsters change
    pub monsters: Option<Vec<MonsterPrototype>>,
    pub monsters_state: Vec<MonsterState>,
}
