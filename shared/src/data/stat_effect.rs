use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

use crate::data::trigger::HitTrigger;

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
}

pub type DamageMap = HashMap<DamageType, (f64, f64)>;

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
    Armor(DamageType),
    TakeFromManaBeforeLife,
    Block,
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
    SpellPower,
    CritChances(#[serde(default)] Option<SkillType>),
    CritDamage(#[serde(default)] Option<SkillType>),
    StatusPower(#[serde(default)] Option<StatStatusType>),
    StatusDuration(#[serde(default)] Option<StatStatusType>),
    Speed(#[serde(default)] Option<SkillType>),
    MovementSpeed,
    GoldFind,
    LifeOnHit(#[serde(default)] HitTrigger),
    ManaOnHit(#[serde(default)] HitTrigger),
    DamageResistance {
        #[serde(default)]
        skill_type: Option<SkillType>,
        #[serde(default)]
        damage_type: Option<DamageType>,
    },
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct StatEffect {
    pub stat: StatType,
    pub modifier: Modifier,
    pub value: f64,
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
                    .entry((target, modifier))
                    .and_modify(|entry| match modifier {
                        Modifier::Flat => *entry += value,
                        Modifier::Multiplier => {
                            let mut new_entry = *entry + 1.0;
                            new_entry.apply_effect(&StatEffect {
                                stat: target,
                                modifier,
                                value,
                            });
                            *entry = new_entry - 1.0;
                        }
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
                    let div = (1.0 - effect.value).max(0.0);
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
            stat: effect.stat,
            modifier: effect.modifier,
            value: -effect.value,
        })
    }
}

impl ApplyStatModifier for f32 {
    fn apply_modifier(&mut self, modifier: Modifier, value: f64) {
        match modifier {
            Modifier::Flat => *self += value as f32,
            Modifier::Multiplier => *self *= (1.0 + value as f32).max(0.0),
        }
    }
}

impl ApplyStatModifier for f64 {
    fn apply_modifier(&mut self, modifier: Modifier, value: f64) {
        match modifier {
            Modifier::Flat => *self += value,
            Modifier::Multiplier => *self *= (1.0 + value).max(0.0),
        }
    }
}
