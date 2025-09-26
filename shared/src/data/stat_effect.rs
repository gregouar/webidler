use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

use crate::data::{
    chance::ChanceRange, character_status::StatusSpecs, skill::RestoreType, trigger::HitTrigger,
};

use super::skill::SkillType;

#[derive(
    Serialize,
    Deserialize,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    Default,
    EnumIter,
)]
pub enum DamageType {
    #[default]
    Physical,
    Fire,
    Poison,
    Storm,
}

pub type DamageMap = HashMap<DamageType, ChanceRange<f64>>;

#[derive(
    Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default,
)]
pub enum Modifier {
    #[default]
    Multiplier,
    Flat,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum StatType {
    Life,
    LifeRegen,
    Mana,
    ManaRegen,
    Armor(Option<DamageType>),
    DamageResistance {
        #[serde(default)]
        skill_type: Option<SkillType>,
        #[serde(default)]
        damage_type: Option<DamageType>,
    },
    TakeFromManaBeforeLife,
    Block,
    BlockSpell,
    BlockDamageTaken,
    Damage {
        #[serde(default)]
        skill_type: Option<SkillType>,
        #[serde(default)]
        damage_type: Option<DamageType>,
    },
    MinDamage {
        #[serde(default)]
        skill_type: Option<SkillType>,
        #[serde(default)]
        damage_type: Option<DamageType>,
    },
    MaxDamage {
        #[serde(default)]
        skill_type: Option<SkillType>,
        #[serde(default)]
        damage_type: Option<DamageType>,
    },
    LifeOnHit(#[serde(default)] HitTrigger),
    ManaOnHit(#[serde(default)] HitTrigger),
    Restore(#[serde(default)] Option<RestoreType>),
    CritChance(#[serde(default)] Option<SkillType>),
    CritDamage(#[serde(default)] Option<SkillType>),
    StatusPower(#[serde(default)] Option<StatStatusType>),
    StatusDuration(#[serde(default)] Option<StatStatusType>),
    Speed(#[serde(default)] Option<SkillType>),
    MovementSpeed,
    GoldFind,
    ThreatGain,
    Lucky {
        #[serde(default)]
        skill_type: Option<SkillType>,
        roll_type: LuckyRollType,
    },
}

fn compare_options<T: PartialEq>(first: &Option<T>, second: &Option<T>) -> bool {
    first.is_none() || second.is_none() || first == second
}

impl StatType {
    pub fn is_match(&self, stat_type: &StatType) -> bool {
        if self == stat_type {
            return true;
        }

        use StatType::*;
        match (self, stat_type) {
            (
                DamageResistance {
                    skill_type,
                    damage_type,
                },
                DamageResistance {
                    skill_type: skill_type_2,
                    damage_type: damage_type_2,
                },
            )
            | (
                Damage {
                    skill_type,
                    damage_type,
                },
                Damage {
                    skill_type: skill_type_2,
                    damage_type: damage_type_2,
                },
            )
            | (
                MinDamage {
                    skill_type,
                    damage_type,
                },
                MinDamage {
                    skill_type: skill_type_2,
                    damage_type: damage_type_2,
                },
            )
            | (
                MaxDamage {
                    skill_type,
                    damage_type,
                },
                MaxDamage {
                    skill_type: skill_type_2,
                    damage_type: damage_type_2,
                },
            ) => {
                compare_options(skill_type, skill_type_2)
                    && compare_options(damage_type, damage_type_2)
            }
            (Restore(first), Restore(second)) => compare_options(first, second),
            (CritChance(first), CritChance(second))
            | (CritDamage(first), CritDamage(second))
            | (Speed(first), Speed(second)) => compare_options(first, second),
            (StatusPower(first), StatusPower(second))
            | (StatusDuration(first), StatusDuration(second)) => match (first, second) {
                (Some(first), Some(second)) => first.is_match(second),
                _ => true,
            },
            _ => false,
        }
    }

    pub fn is_multiplicative(&self) -> bool {
        use StatType::*;

        matches!(
            self,
            Armor(_)
                | Damage { .. }
                | MinDamage { .. }
                | MaxDamage { .. }
                | CritDamage(_)
                | StatusPower(_)
                | GoldFind
        )
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum StatStatusType {
    Stun,
    DamageOverTime {
        #[serde(default)]
        damage_type: Option<DamageType>,
    },
    StatModifier {
        #[serde(default)]
        debuff: Option<bool>,
    },
}

impl StatStatusType {
    pub fn is_match(&self, status_type: &StatStatusType) -> bool {
        if self == status_type {
            return true;
        }

        use StatStatusType::*;
        match (self, status_type) {
            (
                DamageOverTime { damage_type },
                DamageOverTime {
                    damage_type: damage_type_2,
                },
            ) => compare_options(damage_type, damage_type_2),
            (StatModifier { debuff }, StatModifier { debuff: debuff_2 }) => {
                compare_options(debuff, debuff_2)
            }
            _ => false,
        }
    }
}

impl From<&StatusSpecs> for Option<StatStatusType> {
    fn from(value: &StatusSpecs) -> Self {
        match value {
            StatusSpecs::Stun => Some(StatStatusType::Stun),
            StatusSpecs::DamageOverTime { damage_type, .. } => {
                Some(StatStatusType::DamageOverTime {
                    damage_type: Some(*damage_type),
                })
            }
            StatusSpecs::StatModifier { debuff, .. } => Some(StatStatusType::StatModifier {
                debuff: Some(*debuff),
            }),
            StatusSpecs::Trigger(_) => None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum LuckyRollType {
    Damage {
        #[serde(default)]
        damage_type: Option<DamageType>,
    },
    Block,
    CritChance,
    // FailureChance,
    // Restore,
    // StatusDuration,
    // StatusValue,
    // TODO: could add others
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct StatEffect {
    pub stat: StatType,
    pub modifier: Modifier,
    pub value: f64,

    #[serde(default)]
    pub bypass_ignore: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct EffectsMap(pub HashMap<(StatType, Modifier), f64>);

impl From<&EffectsMap> for Vec<StatEffect> {
    fn from(val: &EffectsMap) -> Self {
        val.0
            .iter()
            .map(|((stat, effect_type), value)| StatEffect {
                stat: *stat,
                modifier: *effect_type,
                value: *value,
                bypass_ignore: false,
            })
            .collect()
    }
}

// impl From<Vec<StatEffect>> for EffectsMap {
//     fn from(value: Vec<StatEffect>) -> Self {
//         value.iter().fold(
//             EffectsMap(HashMap::new()),
//             |mut effects_map, stat_effect| {
//                 *effects_map
//                     .0
//                     .entry((stat_effect.stat, stat_effect.modifier))
//                     .or_default() += stat_effect.value;
//                 effects_map
//             },
//         )
//     }
// }

impl EffectsMap {
    pub fn combine_all(maps: impl Iterator<Item = EffectsMap>) -> Self {
        EffectsMap(maps.flat_map(|m| m.0.into_iter()).fold(
            HashMap::new(),
            |mut result, ((target, modifier), value)| {
                result
                    .entry((target, modifier))
                    .and_modify(|entry| match modifier {
                        Modifier::Multiplier if target.is_multiplicative() => {
                            let mut new_entry = *entry + 100.0;
                            new_entry.apply_effect(&StatEffect {
                                stat: target,
                                modifier,
                                value,
                                bypass_ignore: false,
                            });
                            *entry = new_entry - 100.0;
                        }
                        _ => *entry += value,
                    })
                    .or_insert(value);
                result
            },
        ))
    }
}

pub trait ApplyStatModifier {
    fn apply_modifier(&mut self, modifier: Modifier, value: f64);
    fn apply_effect(&mut self, effect: &StatEffect) {
        // We want that negative effect are diminishingly interesting
        let value = match effect.modifier {
            Modifier::Flat => effect.value,
            Modifier::Multiplier => {
                if effect.value >= 0.0 {
                    effect.value
                } else {
                    let div = (1.0 - effect.value * 0.01).max(0.0);
                    if div != 0.0 {
                        effect.value / div
                    } else {
                        0.0
                    }
                }
            }
        };
        self.apply_modifier(effect.modifier, value);
    }

    fn apply_negative_effect(&mut self, effect: &StatEffect) {
        self.apply_effect(&StatEffect {
            value: -effect.value,
            ..*effect
        })
    }
}

impl ApplyStatModifier for f32 {
    fn apply_modifier(&mut self, modifier: Modifier, value: f64) {
        match modifier {
            Modifier::Flat => *self += value as f32,
            Modifier::Multiplier => *self *= (1.0 + value as f32 * 0.01).max(0.0),
        }
    }
}

impl ApplyStatModifier for f64 {
    fn apply_modifier(&mut self, modifier: Modifier, value: f64) {
        match modifier {
            Modifier::Flat => *self += value,
            Modifier::Multiplier => *self *= (1.0 + value * 0.01).max(0.0),
        }
    }
}
