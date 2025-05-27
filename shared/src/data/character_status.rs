use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{skill::DamageType, stat_effect::StatType};

pub type StatusMap = HashMap<StatusType, Vec<StatusState>>;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum StatusType {
    Stun,
    DamageOverTime {
        damage_type: DamageType,
        #[serde(default)]
        ignore_armor: bool,
    },
    StatModifier(StatType),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StatusState {
    pub value: f64,
    pub duration: f64,
    pub cumulate: bool,
}
