use serde::{Deserialize, Serialize};

use super::macros::impl_into_message;

impl_into_message! {
    #[derive(Serialize, Deserialize, Debug, Clone,)]
    pub enum ClientMessage {
        Heartbeat,
        Connect(ClientConnectMessage),
        UseSkill(UseSkillMessage),
        SetAutoSkill(SetAutoSkillMessage),
        LevelUpSkill(LevelUpSkillMessage),
        LevelUpPlayer(LevelUpPlayerMessage),
        EquipItem(EquipItemMessage),
        SellItems(SellItemsMessage),
        PickupLoot(PickUpLootMessage),
    }
}

// Use default to generate heartbeats
impl Default for ClientMessage {
    fn default() -> Self {
        ClientMessage::Heartbeat
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClientConnectMessage {
    pub bearer: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UseSkillMessage {
    pub skill_index: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SetAutoSkillMessage {
    pub skill_index: u8,
    pub auto_use: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LevelUpSkillMessage {
    pub skill_index: u8,
    pub amount: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LevelUpPlayerMessage {
    pub amount: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EquipItemMessage {
    pub item_index: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SellItemsMessage {
    pub item_indexes: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PickUpLootMessage {
    pub loot_identifier: u32,
}
