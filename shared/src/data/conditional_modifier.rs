use serde::{Deserialize, Serialize};

use crate::data::{
    skill::SkillType,
    stat_effect::{StatEffect, StatStatusType},
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
        #[serde(default)]
        status_type: Option<StatStatusType>,
        #[serde(default)]
        skill_type: Option<SkillType>,
        #[serde(default)]
        not: bool,
    },
    StatusStacks {
        #[serde(default)]
        status_type: Option<StatStatusType>,
        #[serde(default)]
        skill_type: Option<SkillType>,
    },
    // StatusValue(Option<StatStatusType>),
    // StatusDuration(Option<StatStatusType>),
    MaximumLife,
    MaximumMana,
    LowLife,
    LowMana,
    ThreatLevel,
}
