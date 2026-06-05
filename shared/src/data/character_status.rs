use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::data::{
    chance::ChanceRange, character::CharacterId, indexed_key::IndexedKey, modifier::Modifier,
    skill::SkillType, trigger::TriggerSpecs, values::NonNegative,
};

use super::{skill::DamageType, stat_effect::StatType};

pub type StatusId = IndexedKey<String>;

pub type StatusMap = HashMap<StatusId, Vec<StatusState>>;

// // pub type StatusMap = HashMap<StatusSpecs, Vec<StatusState>>;
// #[derive(Serialize, Deserialize, Debug, Clone, Default)]
// pub struct StatusMap {
//     // TODO: Do we want to replace this by a map to indexes ?
//     // pub unique_statuses: HashMap<StatusId, usize>, // Points to statuses vec
//     // pub unique_statuses: HashMap<StatusId, (StatusSpecs, StatusState)>,
//     // pub cumulative_statuses: Vec<(StatusSpecs, StatusState)>,
//     pub statuses: HashMap<StatusId, Vec<StatusState>>,
// }

// impl StatusMap {
//     pub fn iter(&self) -> impl Iterator<Item = &(StatusSpecs, StatusState)> {
//         self.unique_statuses
//             .values()
//             .chain(self.cumulative_statuses.iter())
//     }

//     pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut (StatusSpecs, StatusState)> {
//         self.unique_statuses
//             .values_mut()
//             .chain(self.cumulative_statuses.iter_mut())
//     }
// }

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct StatusSpecs {
    pub name: String,
    #[serde(default)]
    pub adjective: Option<String>,
    pub icon: String,

    pub debuff: bool,
    #[serde(default)]
    pub damage_type: Option<DamageType>,
    #[serde(default)]
    pub avoidable: bool,

    pub duration: ChanceRange<NonNegative>,
    pub max_stacks: u8,
    #[serde(default)]
    pub escalation: NonNegative,

    pub effects: Vec<StatusEffect>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct StatusEffect {
    #[serde(flatten)]
    pub status_effect_type: StatusEffectType,
    pub value: NonNegative,
    pub modifier: StatusModifier,
}

impl StatusEffect {
    pub fn computed_value(&self, status_value: NonNegative) -> NonNegative {
        match self.modifier {
            StatusModifier::Flat => self.value,
            StatusModifier::Percent => status_value * (self.value.get() * 0.01),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum StatusEffectType {
    // Stun, // TODO: Replace by statmodifier speed
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
    Trigger {
        #[serde(flatten)]
        trigger_specs: Box<TriggerSpecs>,
        #[serde(default)]
        inherit_owner_effects: bool,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusModifier {
    Flat,
    Percent,
}

// impl StatusSpecs {
//     pub fn into_status_id(&self, skill_type: SkillType) -> StatusId {
//         match self {
//             StatusSpecs::Stun => StatusId::Stun,
//             StatusSpecs::DamageOverTime { damage_type, .. } => StatusId::DamageOverTime {
//                 damage_type: *damage_type,
//             },
//             StatusSpecs::StatModifier {
//                 stat,
//                 modifier,
//                 debuff,
//             } => StatusId::StatModifier {
//                 skill_type,
//                 stat: stat.clone(),
//                 modifier: *modifier,
//                 debuff: *debuff,
//             },
//             StatusSpecs::Trigger(trigger_specs) => {
//                 StatusId::Trigger(trigger_specs.triggered_effect.trigger_id.clone())
//             }
//         }
//     }

//     pub fn is_debuff(&self) -> bool {
//         match self {
//             StatusSpecs::Stun => true,
//             StatusSpecs::DamageOverTime { .. } => true,
//             StatusSpecs::StatModifier { debuff, .. } => *debuff,
//             StatusSpecs::Trigger(trigger_specs) => trigger_specs.is_debuff,
//         }
//     }
// }

// impl From<&StatusSpecs> for StatusId {
//     fn from(val: &StatusSpecs) -> Self {
//         match val {
//             StatusSpecs::Stun => StatusId::Stun,
//             StatusSpecs::DamageOverTime { damage_type, .. } => StatusId::DamageOverTime {
//                 damage_type: *damage_type,
//             },
//             StatusSpecs::StatModifier {
//                 stat,
//                 modifier,
//                 debuff,
//             } => StatusId::StatModifier {
//                 stat: stat.clone(),
//                 modifier: *modifier,
//                 debuff: *debuff,
//             },
//             StatusSpecs::Trigger(trigger_specs) => {
//                 StatusId::Trigger(trigger_specs.triggered_effect.trigger_id.clone())
//             }
//         }
//     }
// }

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StatusState {
    pub owner: CharacterId,

    pub value: NonNegative,
    pub duration: NonNegative, // Remaining duration
    pub skill_type: SkillType, // TODO: Should move to status specs???

    pub base_value: NonNegative,
    pub elapsed_escalation: NonNegative,
    pub max_escalation: NonNegative,
    pub escalation: NonNegative,
}
