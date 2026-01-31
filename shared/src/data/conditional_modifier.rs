use serde::{Deserialize, Serialize};

use crate::data::{stat_effect::StatStatusType, temple::StatEffect};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ConditionalModifier {
    pub conditions: Vec<Condition>,
    pub effects: Vec<StatEffect>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Condition {
    // HitCrit,
    HasStatus(StatStatusType),
    MaximumLife,
    // StatusValue(Option<StatStatusType>),
    // StatusDuration(Option<StatStatusType>),
    // StatusStacks(Option<StatStatusType>),
}
