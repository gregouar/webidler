use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::data::{
    area::{AreaSpecs, AreaState, AreaThreat},
    game_stats::GameStats,
    loot::QueuedLoot,
    monster::{MonsterSpecs, MonsterState},
    passive::{PassivesTreeSpecs, PassivesTreeState},
    player::{PlayerInventory, PlayerResources, PlayerSpecs, PlayerState},
};

use super::macros::impl_into_message;

impl_into_message! {
    #[derive(Serialize, Deserialize, Debug, Clone,)]
    pub enum ServerMessage {
        Connect(ConnectMessage),
        Error(ErrorMessage),
        InitGame(InitGameMessage),
        UpdateGame(SyncGameStateMessage),
        Disconnect(DisconnectMessage),
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConnectMessage {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ErrorMessage {
    pub error_type: ErrorType,
    pub message: String,
    pub must_disconnect: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum ErrorType {
    Server,
    Game,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InitGameMessage {
    pub area_specs: AreaSpecs,
    pub area_state: AreaState,
    pub passives_tree_specs: PassivesTreeSpecs,
    pub passives_tree_state: PassivesTreeState,
    pub player_specs: PlayerSpecs,
    pub player_state: PlayerState,
    pub last_skills_bought: HashSet<String>,
}

/// Message to be sent every tick to sync current state of the game with clients
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SyncGameStateMessage {
    pub area_state: Option<AreaState>,
    pub area_threat: AreaThreat,
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DisconnectMessage {
    pub end_quest: bool,
}
