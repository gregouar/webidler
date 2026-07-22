use serde::{Deserialize, Serialize};

use crate::data::{
    item::{ItemCategory, ItemSlot},
    skill::SkillType,
    stat_effect::{StatEffect, StatStatusFilter},
};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ConditionalModifier {
    pub conditions: Vec<Condition>,
    pub effects: Vec<StatEffect>,

    #[serde(default)]
    pub conditions_duration: u32, // In tenth of second
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Condition {
    // HitCrit,
    HasStatus {
        #[serde(flatten)]
        status_filter: StatStatusFilter,
        #[serde(default)]
        skill_type: Option<SkillType>,
        #[serde(default)]
        not: bool,
    },
    StatusStacks {
        #[serde(flatten)]
        status_filter: StatStatusFilter,
        #[serde(default)]
        skill_type: Option<SkillType>,
    },
    Slowed,
    // StatusValue(Option<StatStatusType>),
    // StatusDuration(Option<StatStatusType>),
    MaximumLife,
    MaximumMana,
    LowLife,
    LowMana,
    ThreatLevel,
    HasItem {
        #[serde(default)]
        item_slot: Option<ItemSlot>,
        #[serde(default)]
        item_category: Option<ItemCategory>,
        #[serde(default)]
        not: bool,
    },
}
