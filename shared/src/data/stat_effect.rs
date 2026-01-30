use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

use crate::data::{
    chance::ChanceRange,
    character_status::StatusSpecs,
    conditional_modifier::Condition,
    skill::{RestoreType, SkillEffectType},
    trigger::HitTrigger,
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum StatType {
    Life,
    LifeRegen,
    Mana,
    ManaRegen,
    ManaCost {
        #[serde(default)]
        skill_type: Option<SkillType>,
    },
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
    // TODO: Collapse to simple Effect, if more involved trigger is needed, we can always add as pure trigger
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
    ItemRarity,
    ThreatGain,
    Lucky {
        #[serde(default)]
        skill_type: Option<SkillType>,
        roll_type: LuckyRollType,
    },
    SkillConditionalModifier {
        stat: Box<StatType>,
        #[serde(default)]
        skill_type: Option<SkillType>,
        #[serde(default)]
        conditions: Vec<Condition>,
    },
    SkillLevel(#[serde(default)] Option<SkillType>),
    StatConverter(StatConverterSpecs),
    StatConditionalModifier {
        stat: Box<StatType>,
        #[serde(default)]
        conditions: Vec<Condition>,
    },
    SuccessChance {
        #[serde(default)]
        skill_type: Option<SkillType>,
        #[serde(default)]
        effect_type: Option<StatSkillEffectType>,
    },
}

pub fn compare_options<T: PartialEq>(first: &Option<T>, second: &Option<T>) -> bool {
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
            (
                Lucky {
                    skill_type,
                    roll_type,
                },
                Lucky {
                    skill_type: skill_type_2,
                    roll_type: roll_type_2,
                },
            ) => compare_options(skill_type, skill_type_2) && roll_type.is_match(roll_type_2),
            (
                SuccessChance {
                    skill_type,
                    effect_type,
                },
                SuccessChance {
                    skill_type: skill_type_2,
                    effect_type: effect_type_2,
                },
            ) => {
                compare_options(skill_type, skill_type_2)
                    && effect_type
                        .zip(*effect_type_2)
                        .is_none_or(|(effect_type, effect_type_2)| {
                            effect_type.is_match(&effect_type_2)
                        })
            }
            (
                ManaCost { skill_type },
                ManaCost {
                    skill_type: skill_type_2,
                },
            ) => compare_options(skill_type, skill_type_2),
            (Restore(first), Restore(second)) => compare_options(first, second),
            (CritChance(first), CritChance(second))
            | (CritDamage(first), CritDamage(second))
            | (Speed(first), Speed(second)) => compare_options(first, second),
            (StatusPower(first), StatusPower(second))
            | (StatusDuration(first), StatusDuration(second)) => match (first, second) {
                (Some(first), Some(second)) => first.is_match(second),
                _ => true,
            },
            (SkillLevel(first), SkillLevel(second)) => compare_options(first, second),
            (
                SkillConditionalModifier {
                    skill_type: first, ..
                },
                SkillConditionalModifier {
                    skill_type: second, ..
                },
            ) => compare_options(first, second),
            _ => false,
        }
    }

    pub fn is_multiplicative(&self) -> bool {
        use StatType::*;

        matches!(
            self,
            Damage { .. }
                | MinDamage { .. }
                | MaxDamage { .. }
                | CritDamage(_)
                | StatusPower(Some(StatStatusType::DamageOverTime { .. }))
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
    Trigger,
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

impl From<&StatusSpecs> for StatStatusType {
    fn from(value: &StatusSpecs) -> Self {
        match value {
            StatusSpecs::Stun => StatStatusType::Stun,
            StatusSpecs::DamageOverTime { damage_type, .. } => StatStatusType::DamageOverTime {
                damage_type: Some(*damage_type),
            },
            StatusSpecs::StatModifier { debuff, .. } => StatStatusType::StatModifier {
                debuff: Some(*debuff),
            },
            StatusSpecs::Trigger(_) => StatStatusType::Trigger,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum StatSkillEffectType {
    FlatDamage {
        // damage_type: Option<DamageType>,
    },
    ApplyStatus {
        // status_type: Option<StatStatusType>,
    },
    Restore {
        #[serde(default)]
        restore_type: Option<RestoreType>,
    },
    Resurrect,
}

impl StatSkillEffectType {
    pub fn is_match(&self, skill_effect_type: &StatSkillEffectType) -> bool {
        if self == skill_effect_type {
            return true;
        }

        use StatSkillEffectType::*;
        match (self, skill_effect_type) {
            (
                Restore { restore_type },
                Restore {
                    restore_type: restore_type_2,
                },
            ) => compare_options(restore_type, restore_type_2),
            _ => false,
        }
    }
}

impl From<&SkillEffectType> for Option<StatSkillEffectType> {
    fn from(value: &SkillEffectType) -> Self {
        match value {
            SkillEffectType::FlatDamage { .. } => Some(StatSkillEffectType::FlatDamage {}),
            SkillEffectType::ApplyStatus { .. } => Some(StatSkillEffectType::ApplyStatus {}),
            SkillEffectType::Restore { restore_type, .. } => Some(StatSkillEffectType::Restore {
                restore_type: Some(*restore_type),
            }),
            SkillEffectType::Resurrect => Some(StatSkillEffectType::Resurrect),
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
    SuccessChance,
    // Restore,
    // StatusDuration,
    // StatusValue,
    // TODO: could add others
}

impl LuckyRollType {
    pub fn is_match(&self, lucky_roll_type: &LuckyRollType) -> bool {
        if self == lucky_roll_type {
            return true;
        }

        use LuckyRollType::*;
        match (self, lucky_roll_type) {
            (
                Damage { damage_type },
                Damage {
                    damage_type: damage_type_2,
                },
            ) => compare_options(damage_type, damage_type_2),
            _ => false,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct StatConverterSpecs {
    pub source: StatConverterSource,
    pub target_stat: Box<StatType>,
    pub target_modifier: Modifier,

    #[serde(default)]
    pub is_extra: bool,
    #[serde(default)]
    pub skill_type: Option<SkillType>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum StatConverterSource {
    CritDamage,
    Damage {
        #[serde(default)]
        damage_type: Option<DamageType>,
    },
    ThreatLevel,
    MaxLife,
    MaxMana,
    ManaRegen,
    LifeRegen,
    // TODO: Add others, like armor, ...
}

fn is_false(value: &bool) -> bool {
    !*value
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct StatEffect {
    pub stat: StatType,
    pub modifier: Modifier,
    pub value: f64,

    #[serde(default, skip_serializing_if = "is_false")]
    pub bypass_ignore: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct EffectsMap(pub HashMap<(StatType, Modifier), f64>);

impl From<&EffectsMap> for Vec<StatEffect> {
    fn from(val: &EffectsMap) -> Self {
        val.0
            .iter()
            .map(|((stat, effect_type), value)| StatEffect {
                stat: stat.clone(),
                modifier: *effect_type,
                value: *value,
                bypass_ignore: false,
            })
            .collect()
    }
}

impl EffectsMap {
    pub fn combine_all(maps: impl Iterator<Item = EffectsMap>) -> Self {
        EffectsMap(maps.flat_map(|m| m.0.into_iter()).fold(
            HashMap::new(),
            |mut result, ((target, modifier), value)| {
                result
                    .entry((target.clone(), modifier))
                    .and_modify(|entry| match modifier {
                        Modifier::Multiplier if target.is_multiplicative() => {
                            if *entry == 0.0 {
                                *entry = value;
                                return;
                            }

                            let sign = if *entry < 0.0 { -1.0 } else { 1.0 };
                            let mut new_entry = entry.abs() + 100.0;

                            new_entry.apply_effect(&StatEffect {
                                stat: target,
                                modifier,
                                value: sign * value,
                                bypass_ignore: false,
                            });

                            *entry = sign
                                * if new_entry >= 100.0 {
                                    new_entry - 100.0
                                } else {
                                    -100.0 / (new_entry * 0.01) + 100.0
                                };
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

                    if effect.value <= -1e300 {
                        -100.0
                    } else if div != 0.0 {
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
            ..effect.clone()
        })
    }
}

impl ApplyStatModifier for f32 {
    fn apply_modifier(&mut self, modifier: Modifier, value: f64) {
        match modifier {
            Modifier::Flat => *self += value as f32,
            Modifier::Multiplier => {
                //TODO: Check if this is OK, the idea was that inc restore shouldn't apply if already negative
                if *self > 0.0 {
                    *self *= (100.0 + value as f32).max(0.0) * 0.01
                }
            }
        }
    }
}

impl ApplyStatModifier for f64 {
    fn apply_modifier(&mut self, modifier: Modifier, value: f64) {
        match modifier {
            Modifier::Flat => *self += value,
            Modifier::Multiplier => {
                if *self > 0.0 {
                    *self *= (100.0 + value).max(0.0) * 0.01
                }
            }
        }
    }
}
