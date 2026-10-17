use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::data::{
    area::AreaLevel,
    item::{ItemCategory, ItemRarity, ItemSlot},
    skill::SkillType,
};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct AchievementSpecs {
    // pub scope: AchievementScope, Could be User, Character, Realm etc later
    pub icon: String,
    pub name: String,
    pub description: String,

    pub goals: Vec<AchievementGoal>,
    #[serde(default)]
    pub goals_amount: Option<u8>,

    pub rewards: Vec<AchievementReward>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum AchievementReward {
    Cosmetic(String),
    Pet(String),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum AchievementGoal {
    AreaLevel {
        value: AreaLevel,
        #[serde(default)]
        area_id: Option<String>,
    },
    PowerLevel(AreaLevel),
    PlayerLevel(u8),
    SkillLevel {
        value: u8,
        #[serde(default)]
        skill_id: Option<String>,
        #[serde(default)]
        skill_type: Option<SkillType>,
        #[serde(default = "default_one")]
        amount: u8,
    },
    PassiveLevel {
        value: u8,
        #[serde(default)]
        passive_id: Option<Uuid>,
        #[serde(default = "default_one")]
        amount: u8,
    },
    EquippedItem {
        #[serde(default)]
        item_id: Option<String>,
        #[serde(default)]
        item_slot: Option<ItemSlot>,
        #[serde(default)]
        item_level: Option<AreaLevel>,
        #[serde(default)]
        item_rarity: Option<ItemRarity>,
        #[serde(default)]
        item_category: Option<ItemCategory>,
        #[serde(default)]
        in_passives_tree: bool,
        #[serde(default = "default_one")]
        amount: u8,
    },
}

fn default_one() -> u8 {
    1
}
