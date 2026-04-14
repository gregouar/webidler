use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

use crate::data::{
    chance::ChanceRange,
    character_status::StatusSpecs,
    conditional_modifier::Condition,
    item::{SkillRange, SkillShape},
    modifier::{ModifiableValue, Modifier, compute_more_factor},
    skill::{RestoreType, SkillEffectType, SkillRepeatTarget},
    values::NonNegative,
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

impl Matchable for DamageType {
    fn is_match(&self, other: &Self) -> bool {
        *self == *other
    }
}

pub type DamageMap = BTreeMap<DamageType, ChanceRange<ModifiableValue<NonNegative>>>;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum MinMax {
    Min,
    Max,
}

impl Matchable for MinMax {
    fn is_match(&self, other: &Self) -> bool {
        *self == *other
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum StatType {
    Description(String),
    GemsFind,
    ItemRarity,
    ItemLevel,
    SkillLevel(#[serde(default)] Option<SkillType>),
    Armor(Option<ArmorStatType>),
    DamageResistance {
        #[serde(default)]
        skill_type: Option<SkillType>,
        #[serde(default)]
        damage_type: Option<DamageType>,
    },
    Block(#[serde(default)] Option<SkillType>),
    BlockDamageTaken,
    Evade(#[serde(default)] Option<DamageType>),
    EvadeDamageTaken,
    StatusResistance {
        #[serde(default)]
        skill_type: Option<SkillType>,
        #[serde(default)]
        status_type: Option<StatStatusType>,
    },
    Damage {
        #[serde(default)]
        skill_type: Option<SkillType>,
        #[serde(default)]
        damage_type: Option<DamageType>,
        #[serde(default)]
        min_max: Option<MinMax>,
    },
    CritChance(#[serde(default)] Option<SkillType>),
    CritDamage(#[serde(default)] Option<SkillType>),
    StatusPower {
        #[serde(default)]
        status_type: Option<StatStatusType>,
        #[serde(default)]
        skill_type: Option<SkillType>,
        #[serde(default)]
        min_max: Option<MinMax>,
    },
    StatusDuration {
        #[serde(default)]
        status_type: Option<StatStatusType>,
        #[serde(default)]
        skill_type: Option<SkillType>,
    },
    SuccessChance {
        #[serde(default)]
        skill_type: Option<SkillType>,
        #[serde(default)]
        effect_type: Option<StatSkillEffectType>,
    },
    Speed(#[serde(default)] Option<SkillType>),
    RestoreOnHit {
        restore_type: RestoreType,
        #[serde(default)]
        skill_type: Option<SkillType>,
    },
    Restore {
        #[serde(default)]
        restore_type: Option<RestoreType>,
        #[serde(default)]
        skill_type: Option<SkillType>,
    },
    Life,
    LifeRegen,
    Mana,
    ManaRegen,
    ManaCost {
        #[serde(default)]
        skill_type: Option<SkillType>,
    },
    TakeFromManaBeforeLife,
    TakeFromLifeBeforeMana,
    MovementSpeed,
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
    StatConditionalModifier {
        stat: Box<StatType>,
        conditions: Vec<Condition>,
        #[serde(default)]
        conditions_duration: u32,
    },
    StatConverter(StatConverterSpecs),
    SkillTargetModifier {
        // TODO: More control and options?
        #[serde(default)]
        skill_type: Option<SkillType>,
        #[serde(default)]
        range: Option<SkillRange>,
        #[serde(default)]
        shape: Option<SkillShape>,
        #[serde(default)]
        repeat: Option<StatSkillRepeat>,
        #[serde(default)]
        skill_id: Option<String>,
    },
    GoldFind,
    PowerLevel,
    Description2(String),
}

impl Matchable for StatType {
    fn is_match(&self, stat_type: &StatType) -> bool {
        use StatType::*;
        match (self, stat_type) {
            (
                Damage {
                    skill_type,
                    damage_type,
                    min_max,
                },
                Damage {
                    skill_type: skill_type_2,
                    damage_type: damage_type_2,
                    min_max: min_max_2,
                },
            ) => {
                compare_options(skill_type, skill_type_2)
                    && compare_options(damage_type, damage_type_2)
                    && compare_options(min_max, min_max_2)
            }
            (
                DamageResistance {
                    skill_type,
                    damage_type,
                },
                DamageResistance {
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
                    && compare_options(effect_type, effect_type_2)
            }
            (
                ManaCost { skill_type },
                ManaCost {
                    skill_type: skill_type_2,
                },
            ) => compare_options(skill_type, skill_type_2),
            (
                Restore {
                    restore_type,
                    skill_type,
                },
                Restore {
                    restore_type: restore_type_2,
                    skill_type: skill_type_2,
                },
            ) => {
                compare_options(restore_type, restore_type_2)
                    && compare_options(skill_type, skill_type_2)
            }
            (CritChance(first), CritChance(second))
            | (CritDamage(first), CritDamage(second))
            | (Speed(first), Speed(second))
            | (Block(first), Block(second)) => compare_options(first, second),
            (Evade(first), Evade(second)) => compare_options(first, second),
            (
                StatusPower {
                    status_type,
                    skill_type,
                    min_max,
                },
                StatusPower {
                    status_type: status_type_2,
                    skill_type: skill_type_2,
                    min_max: min_max_2,
                },
            ) => {
                compare_options(status_type, status_type_2)
                    && compare_options(skill_type, skill_type_2)
                    && compare_options(min_max, min_max_2)
            }
            (
                StatusDuration {
                    status_type,
                    skill_type,
                },
                StatusDuration {
                    status_type: status_type_2,
                    skill_type: skill_type_2,
                },
            )
            | (
                StatusResistance {
                    status_type,
                    skill_type,
                },
                StatusResistance {
                    status_type: status_type_2,
                    skill_type: skill_type_2,
                },
            ) => {
                compare_options(status_type, status_type_2)
                    && compare_options(skill_type, skill_type_2)
            }
            (SkillLevel(first), SkillLevel(second)) => compare_options(first, second),
            (
                SkillConditionalModifier {
                    skill_type: first, ..
                },
                SkillConditionalModifier {
                    skill_type: second, ..
                },
            ) => compare_options(first, second),
            _ => self == stat_type,
        }
    }

    // pub fn is_multiplicative(&self) -> bool {
    //     use StatType::*;

    //     matches!(
    //         self,
    //         Damage { .. }
    //             | CritDamage(_)
    //             | StatusPower {
    //                 status_type: Some(StatStatusType::DamageOverTime { .. }),
    //                 ..
    //             }
    //             | GoldFind
    //     )
    // }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct StatSkillRepeat {
    pub min_value: u8,
    pub max_value: u8,
    pub target: SkillRepeatTarget,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ArmorStatType {
    Physical,
    Fire,
    Poison,
    Storm,
    Elemental,
}

impl ArmorStatType {
    pub fn is_match(&self, damage_type: DamageType) -> bool {
        match (self, damage_type) {
            (ArmorStatType::Physical, DamageType::Physical) => true,
            (ArmorStatType::Fire | ArmorStatType::Elemental, DamageType::Fire) => true,
            (ArmorStatType::Poison | ArmorStatType::Elemental, DamageType::Poison) => true,
            (ArmorStatType::Storm | ArmorStatType::Elemental, DamageType::Storm) => true,
            _ => false,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum StatStatusType {
    Stun,
    DamageOverTime {
        #[serde(default)]
        damage_type: Option<DamageType>,
    },
    StatModifier {
        #[serde(default)]
        debuff: Option<bool>,
        #[serde(default)]
        stat: Option<Box<StatType>>,
    },
    Trigger {
        #[serde(default)]
        trigger_id: Option<String>,

        // TODO: This is awful....
        #[serde(default)]
        trigger_description: Option<String>,
    },
}

impl Matchable for StatStatusType {
    fn is_match(&self, status_type: &StatStatusType) -> bool {
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
            (
                StatModifier { debuff, stat },
                StatModifier {
                    debuff: debuff_2,
                    stat: stat2,
                },
            ) => {
                compare_options(debuff, debuff_2)
                    && compare_options(&stat.as_deref(), &stat2.as_deref())
            }
            (
                Trigger {
                    trigger_id,
                    trigger_description: _,
                },
                Trigger {
                    trigger_id: trigger_id_2,
                    trigger_description: _,
                },
            ) => compare_options(trigger_id, trigger_id_2),
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
            StatusSpecs::StatModifier { debuff, stat, .. } => StatStatusType::StatModifier {
                debuff: Some(*debuff),
                stat: Some(stat.clone().into()),
            },
            StatusSpecs::Trigger(trigger_specs) => StatStatusType::Trigger {
                trigger_id: Some(trigger_specs.triggered_effect.trigger_id.clone()),
                trigger_description: None,
            },
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum StatSkillEffectType {
    FlatDamage {
        // damage_type: Option<DamageType>,
    },
    ApplyStatus {
        status_type: Option<StatStatusType>,
    },
    Restore {
        #[serde(default)]
        restore_type: Option<RestoreType>,
    },
    Resurrect,
}

impl Matchable for StatSkillEffectType {
    fn is_match(&self, skill_effect_type: &StatSkillEffectType) -> bool {
        use StatSkillEffectType::*;
        match (self, skill_effect_type) {
            (
                Restore { restore_type },
                Restore {
                    restore_type: restore_type_2,
                },
            ) => compare_options(restore_type, restore_type_2),
            (
                ApplyStatus { status_type },
                ApplyStatus {
                    status_type: status_type_2,
                },
            ) => compare_options(status_type, status_type_2),
            _ => self == skill_effect_type,
        }
    }
}

impl From<&SkillEffectType> for Option<StatSkillEffectType> {
    fn from(value: &SkillEffectType) -> Self {
        match value {
            SkillEffectType::FlatDamage { .. } => Some(StatSkillEffectType::FlatDamage {}),
            SkillEffectType::ApplyStatus { statuses, .. } => {
                Some(StatSkillEffectType::ApplyStatus {
                    status_type: statuses.first().map(|status| (&status.status_type).into()),
                })
            }
            SkillEffectType::Restore { restore_type, .. } => Some(StatSkillEffectType::Restore {
                restore_type: Some(*restore_type),
            }),
            SkillEffectType::Resurrect => Some(StatSkillEffectType::Resurrect),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum LuckyRollType {
    Damage {
        #[serde(default)]
        damage_type: Option<DamageType>,
    },
    Block,
    Evade(Option<DamageType>),
    CritChance,
    SuccessChance {
        #[serde(default)]
        effect_type: Option<StatSkillEffectType>,
    },
    // Restore,
    // StatusDuration,
    // StatusValue,
    // TODO: could add others
}

impl Matchable for LuckyRollType {
    fn is_match(&self, lucky_roll_type: &LuckyRollType) -> bool {
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
            (
                SuccessChance { effect_type },
                SuccessChance {
                    effect_type: effect_type_2,
                },
            ) => compare_options(effect_type, effect_type_2),
            _ => false,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct StatConverterSpecs {
    pub source: StatConverterSource,
    pub stat: Box<StatType>,

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
        #[serde(default)]
        min_max: Option<MinMax>,
    },
    // DamageOverTime {
    //     #[serde(default)]
    //     damage_type: Option<DamageType>,
    //     #[serde(default)]
    //     min_max: Option<MinMax>,
    // },
    // ThreatLevel,
    MaxLife,
    MaxMana,
    ManaRegen,
    LifeRegen,
    Block(SkillType),
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

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct EffectsMap(pub HashMap<(StatType, Modifier, bool), f64>);

impl From<&EffectsMap> for Vec<StatEffect> {
    fn from(val: &EffectsMap) -> Self {
        val.0
            .iter()
            .map(|((stat, effect_type, bypass_ignore), value)| StatEffect {
                stat: stat.clone(),
                modifier: *effect_type,
                value: *value,
                bypass_ignore: *bypass_ignore,
            })
            .collect()
    }
}

impl From<Vec<StatEffect>> for EffectsMap {
    fn from(value: Vec<StatEffect>) -> Self {
        EffectsMap::combine_all(value.into_iter().map(|x| {
            EffectsMap(HashMap::from([(
                (x.stat, x.modifier, x.bypass_ignore),
                x.value,
            )]))
        }))
    }
}

impl From<Vec<&StatEffect>> for EffectsMap {
    fn from(value: Vec<&StatEffect>) -> Self {
        EffectsMap::combine_all(value.iter().map(|x| {
            EffectsMap(HashMap::from([(
                (x.stat.clone(), x.modifier, x.bypass_ignore),
                x.value,
            )]))
        }))
    }
}

impl EffectsMap {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn combine_all(maps: impl Iterator<Item = EffectsMap>) -> Self {
        EffectsMap(maps.flat_map(|m| m.0.into_iter()).fold(
            HashMap::new(),
            |mut result, ((target, modifier, bypass_ignore), value)| {
                result
                    .entry((target.clone(), modifier, bypass_ignore))
                    .and_modify(|entry| match modifier {
                        Modifier::More => {
                            if *entry == 0.0 {
                                *entry = value;
                                return;
                            }

                            let sign = if *entry < 0.0 { -1.0 } else { 1.0 };

                            let factor = compute_more_factor(sign * value);
                            *entry = sign
                                * compute_more_factor(
                                    entry.abs() + factor + entry.abs() * factor * 0.01,
                                );
                        }
                        _ => *entry += value,
                    })
                    .or_insert(value);
                result
            },
        ))
    }

    pub fn iter(&self) -> impl Iterator<Item = StatEffect> {
        self.0
            .iter()
            .map(|((stat, effect_type, bypass_ignore), value)| StatEffect {
                stat: stat.clone(),
                modifier: *effect_type,
                value: *value,
                bypass_ignore: *bypass_ignore,
            })
    }
}

pub trait Matchable {
    fn is_match(&self, other: &Self) -> bool;
}

impl<T: Matchable> Matchable for &T {
    fn is_match(&self, other: &Self) -> bool {
        (*self).is_match(*other)
    }
}

impl Matchable for bool {
    fn is_match(&self, other: &Self) -> bool {
        *self == *other
    }
}

impl Matchable for String {
    fn is_match(&self, other: &Self) -> bool {
        *self == *other
    }
}

pub fn compare_options<T: Matchable>(first: &Option<T>, second: &Option<T>) -> bool {
    match (first, second) {
        (None, _) | (_, None) => true,
        (Some(a), Some(b)) => a.is_match(b),
    }
}
