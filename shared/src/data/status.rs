use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::skill::DamageType;

pub type StatusMap = HashMap<StatusType, StatusState>;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum StatusType {
    Stunned,
    DamageOverTime(DamageType),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StatusState {
    pub value: f64,
    pub duration: f64,
}
