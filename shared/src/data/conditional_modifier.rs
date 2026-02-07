use serde::{Deserialize, Serialize};

use crate::data::{skill::SkillType, stat_effect::StatStatusType, temple::StatEffect};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ConditionalModifier {
    pub conditions: Vec<Condition>,
    pub effects: Vec<StatEffect>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Condition {
    // HitCrit,
    HasStatus {
        #[serde(default)]
        status_type: Option<StatStatusType>,
        #[serde(default)]
        skill_type: Option<SkillType>,
    },
    MaximumLife,
    MaximumMana,
    // StatusValue(Option<StatStatusType>),
    // StatusDuration(Option<StatStatusType>),
    // StatusStacks(Option<StatStatusType>),
}
