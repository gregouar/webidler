use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::data::{modifier::Modifier, skill::SkillType, trigger::TriggerSpecs};

use super::{skill::DamageType, stat_effect::StatType};

// pub type StatusMap = HashMap<StatusSpecs, Vec<StatusState>>;
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct StatusMap {
    // TODO: Do we want to replace this by a map to indexes ?
    // pub unique_statuses: HashMap<StatusId, usize>, // Points to statuses vec
    pub unique_statuses: HashMap<(StatusId, SkillType), (StatusSpecs, StatusState)>,
    pub cumulative_statuses: Vec<(StatusSpecs, StatusState)>,
}

impl StatusMap {
    pub fn iter(&self) -> impl Iterator<Item = &(StatusSpecs, StatusState)> {
        self.unique_statuses
            .values()
            .chain(self.cumulative_statuses.iter())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum StatusId {
    Stun,
    DamageOverTime {
        damage_type: DamageType,
    },
    StatModifier {
        stat: StatType,
        modifier: Modifier,
        debuff: bool,
    },
    Trigger(String),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum StatusSpecs {
    Stun,
    DamageOverTime {
        damage_type: DamageType,
    },
    StatModifier {
        stat: StatType,
        #[serde(default)]
        modifier: Modifier,
        #[serde(default)]
        debuff: bool,
    },
    Trigger(Box<TriggerSpecs>),
}

impl From<&StatusSpecs> for StatusId {
    fn from(val: &StatusSpecs) -> Self {
        match val {
            StatusSpecs::Stun => StatusId::Stun,
            StatusSpecs::DamageOverTime { damage_type, .. } => StatusId::DamageOverTime {
                damage_type: *damage_type,
            },
            StatusSpecs::StatModifier {
                stat,
                modifier,
                debuff,
            } => StatusId::StatModifier {
                stat: stat.clone(),
                modifier: *modifier,
                debuff: *debuff,
            },
            StatusSpecs::Trigger(trigger_specs) => {
                StatusId::Trigger(trigger_specs.trigger_id.clone())
            }
        }
    }
}

// IDEA: Separate more status specs from status id, do hash map of status id + cumulable
// Use skill_id as status_id for special buffs and default status_id for others (status_id would be enum BaseStatusTYpe + String ?)

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum StatModifierType {
    Buff,
    Debuff,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StatusState {
    pub value: f64,
    pub duration: Option<f64>,
    pub cumulate: bool,
    pub skill_type: SkillType,
}
