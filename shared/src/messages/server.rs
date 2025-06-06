use serde::{Deserialize, Serialize};

use crate::data::{
    game_stats::GameStats,
    loot::QueuedLoot,
    monster::{MonsterSpecs, MonsterState},
    passive::{PassivesTreeSpecs, PassivesTreeState},
    player::{PlayerInventory, PlayerResources, PlayerSpecs, PlayerState},
    world::{WorldSpecs, WorldState},
};

use super::{macros::impl_into_message, SessionId, SessionKey};

impl_into_message! {
    #[derive(Serialize, Deserialize, Debug, Clone,)]
    pub enum ServerMessage {
        Connect(ConnectMessage),
        Error(ErrorMessage),
        InitGame(InitGameMessage),
        UpdateGame(SyncGameStateMessage),
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConnectMessage {
    pub session_id: SessionId,
    pub session_key: SessionKey,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ErrorMessage {
    pub error_type: ErrorType,
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum ErrorType {
    Server,
    Game,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InitGameMessage {
    pub world_specs: WorldSpecs,
    pub world_state: WorldState,
    pub passives_tree_specs: PassivesTreeSpecs,
    pub passives_tree_state: PassivesTreeState,
    pub player_specs: PlayerSpecs,
    pub player_state: PlayerState,
}

/// Message to be sent every tick to sync current state of the game with clients
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SyncGameStateMessage {
    pub world_state: Option<WorldState>,
    pub passives_tree_state: Option<PassivesTreeState>,
    pub player_specs: Option<PlayerSpecs>,
    pub player_inventory: Option<PlayerInventory>,
    pub player_state: PlayerState,
    pub player_resources: Option<PlayerResources>,
    pub monster_specs: Option<Vec<MonsterSpecs>>,
    pub monster_states: Vec<MonsterState>,
    pub queued_loot: Option<Vec<QueuedLoot>>,
    pub game_stats: GameStats,
}
