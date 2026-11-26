use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    data::{
        area::AreaLevel,
        character_status::StatusId,
        item::{BaseItemId, ItemCategory, ItemRarity, ItemSlot},
        monster::MonsterRarity,
        skill::{SkillType, TargetType},
    },
    serde_utils::default_one,
};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AchievementCounter {
    GambleUnique,
    ImbueUniqueSoul,
    SacrificeEquipment600,
    AddAffix600,
    GrindCompleted,
    UniqueGrindCompleted,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct AchievementSpecs {
    // pub scope: AchievementScope, Could be User, Character, Realm etc later
    pub category: AchievementCategory,

    pub icon: String,
    pub name: String,
    pub description: String,

    pub goals: Vec<AchievementGoal>,
    #[serde(default)]
    pub goals_amount: Option<u8>,

    pub rewards: Vec<AchievementReward>,
}

impl AchievementSpecs {
    pub fn required_goal_count(&self) -> u64 {
        self.goals_amount.map(u64::from).unwrap_or_else(|| {
            self.goals
                .iter()
                .map(AchievementGoal::required_amount)
                .sum()
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AchievementCategory {
    Global,
    Grind,
    // Exotic
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum AchievementReward {
    Cosmetic(String),
    Pet(String),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum AchievementGoal {
    Counter {
        counter: AchievementCounter,
        value: u64,
    },
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
    PassiveAllocated {
        amount: u8,
        #[serde(default)]
        socket: bool,
    },
    EquippedItem {
        #[serde(default)]
        item_id: Option<BaseItemId>,
        #[serde(default)]
        item_slot: Option<ItemSlot>,
        #[serde(default)]
        item_level: Option<AreaLevel>,
        #[serde(default)]
        item_quality: Option<f32>,
        #[serde(default)]
        item_rarity: Option<ItemRarity>,
        #[serde(default)]
        item_category: Option<ItemCategory>,
        #[serde(default)]
        in_passives_tree: bool,
        #[serde(default = "default_one")]
        amount: u8,
    },
    MonstersKilled {
        value: u64,
        #[serde(default)]
        monster_rarity: Option<MonsterRarity>,
        #[serde(default)]
        area_id: Option<String>,
    },
    AreaCompleted {
        value: u64,
        #[serde(default)]
        area_id: Option<String>,
        #[serde(default)]
        with_edict: bool,
    },
    GemsObtained {
        value: u64,
        #[serde(default)]
        area_id: Option<String>,
    },
    CharacterStatusStacks {
        value: u64,
        status_id: StatusId,
        target: TargetType,
    },
    PlayerDeath {
        value: u64,
        #[serde(default)]
        area_id: Option<String>,
    },
}

impl AchievementGoal {
    pub fn required_amount(&self) -> u64 {
        match self {
            Self::SkillLevel { amount, .. }
            | Self::PassiveLevel { amount, .. }
            | Self::PassiveAllocated { amount, .. }
            | Self::EquippedItem { amount, .. } => u64::from(*amount),
            Self::MonstersKilled { value, .. }
            | Self::AreaCompleted { value, .. }
            | Self::GemsObtained { value, .. }
            | Self::PlayerDeath { value, .. }
            | Self::Counter { value, .. }
            | Self::CharacterStatusStacks { value, .. } => *value,
            Self::AreaLevel { .. } | Self::PowerLevel(_) | Self::PlayerLevel(_) => 1,
        }
    }
}
