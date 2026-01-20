use serde::{Deserialize, Serialize};

use crate::data::{
    area::{AreaLevel, StartAreaConfig},
    item::{ItemCategory, ItemSlot},
    passive::PassiveNodeId,
    user::UserCharacterId,
};

use super::macros::impl_into_message;

impl_into_message! {
    #[derive(Serialize, Deserialize, Debug, Clone,)]
    pub enum ClientMessage {
        Heartbeat,

        Connect(ClientConnectMessage),

        EndQuest,

        UseSkill(UseSkillMessage),
        SetAutoSkill(SetAutoSkillMessage),
        LevelUpSkill(LevelUpSkillMessage),
        BuySkill(BuySkillMessage),

        LevelUpPlayer(LevelUpPlayerMessage),
        PurchasePassive(PurchasePassiveMessage),

        EquipItem(EquipItemMessage),
        UnequipItem(UnequipItemMessage),
        SellItems(SellItemsMessage),

        FilterLoot(FilterLootMessage),
        PickupLoot(PickUpLootMessage),

        SetAutoProgress(SetAutoProgressMessage),
        GoBack(GoBackLevelMessage),
        SetRushMode(SetRushModeMessage),
    }
}

// Use default to generate heartbeats
#[allow(clippy::derivable_impls)]
impl Default for ClientMessage {
    fn default() -> Self {
        ClientMessage::Heartbeat
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClientConnectMessage {
    pub jwt: String,
    pub character_id: UserCharacterId,
    pub area_config: Option<StartAreaConfig>,
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
pub struct BuySkillMessage {
    pub skill_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LevelUpPlayerMessage {
    pub amount: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PurchasePassiveMessage {
    pub node_id: PassiveNodeId,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EquipItemMessage {
    pub item_index: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UnequipItemMessage {
    pub item_slot: ItemSlot,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SellItemsMessage {
    pub item_indexes: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FilterLootMessage {
    pub preferred_loot: Option<ItemCategory>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PickUpLootMessage {
    pub loot_identifier: u32,
    pub sell: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GoBackLevelMessage {
    pub amount: AreaLevel,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SetAutoProgressMessage {
    pub value: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SetRushModeMessage {
    pub value: bool,
}
