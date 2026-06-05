use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

use crate::{
    data::{
        chance::ChanceRange,
        character_status::StatusId,
        conditional_modifier::Condition,
        item::{SkillRange, SkillShape},
        modifier::{ModifiableValue, Modifier, compute_more_factor},
        skill::{RestoreType, SkillEffectType, SkillRepeatTarget},
        values::NonNegative,
    },
    serde_utils::is_false,
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

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct StatSkillFilter {
    #[serde(default)]
    pub skill_type: Option<SkillType>,
    #[serde(default)]
    pub skill_id: Option<String>,
}

impl Matchable for StatSkillFilter {
    fn is_match(&self, second: &StatSkillFilter) -> bool {
        compare_options(&self.skill_type, &second.skill_type)
            && compare_options(&self.skill_id, &second.skill_id)
    }
}

impl StatSkillFilter {
    pub fn is_match_with_skill(&self, skill_type: SkillType, skill_id: &String) -> bool {
        compare_options(&self.skill_type, &Some(skill_type))
            && compare_options(&self.skill_id.as_ref(), &Some(skill_id))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum StatusDamageType {
    Any,
    Physical,
    Fire,
    Poison,
    Storm,
}

impl Matchable for StatusDamageType {
    fn is_match(&self, other: &Self) -> bool {
        use StatusDamageType::*;
        match (self, other) {
            (Any, _) | (_, Any) => true,
            (x, y) if x == y => true,
            _ => false,
        }
    }
}

impl From<DamageType> for StatusDamageType {
    fn from(value: DamageType) -> Self {
        use StatusDamageType::*;
        match value {
            DamageType::Physical => Physical,
            DamageType::Fire => Fire,
            DamageType::Poison => Poison,
            DamageType::Storm => Storm,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct StatStatusFilter {
    #[serde(default)]
    pub status_id: Option<StatusId>,
    #[serde(default)]
    pub damage_type: Option<StatusDamageType>,
}

impl Matchable for StatStatusFilter {
    fn is_match(&self, second: &StatStatusFilter) -> bool {
        compare_options(&self.status_id, &second.status_id)
            && match (self.damage_type, second.damage_type) {
                (None, None) => true,
                (None, Some(_)) | (Some(_), None) => false,
                (Some(damage_type), Some(damage_type_2)) => damage_type.is_match(&damage_type_2),
            }
    }
}

impl StatStatusFilter {
    pub fn is_match_with_status(
        &self,
        status_id: &StatusId,
        damage_type: Option<DamageType>,
    ) -> bool {
        self.status_id
            .as_ref()
            .map(|filter_status_id| filter_status_id == status_id.as_str())
            .unwrap_or(true)
            && self
                .damage_type
                .as_ref()
                .map(|filter_damage_type| match damage_type {
                    None => false, // not damage over time
                    Some(damage_type) => filter_damage_type.is_match(&damage_type.into()),
                })
                .unwrap_or(true)
    }
}

// impl SkillFilterMatchable for Option<StatSkillFilter> {
//     fn is_match_with_skill(&self, skill_type: SkillType, skill_id: &String) -> bool {
//         self.as_ref()
//             .map(|filter| filter.is_match_with_skill(skill_type, skill_id))
//             .unwrap_or(true)
//     }
// }

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum StatType {
    Description(String),
    GemsFind,
    ItemRarity,
    ItemLevel,
    SkillLevel(#[serde(default)] StatSkillFilter),
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
        status_id: Option<StatusId>,
        #[serde(default)]
        skill_type: Option<SkillType>,
    },
    Damage {
        #[serde(flatten)]
        skill_filter: StatSkillFilter,
        #[serde(default)]
        damage_type: Option<DamageType>,
        #[serde(default)]
        min_max: Option<MinMax>,
        #[serde(default)]
        is_hit: Option<bool>,
    },
    CritChance(#[serde(default)] StatSkillFilter),
    CritDamage(#[serde(default)] StatSkillFilter),
    StatusPower {
        #[serde(flatten)]
        status_filter: StatStatusFilter,
        #[serde(flatten)]
        skill_filter: StatSkillFilter,
        #[serde(default)]
        min_max: Option<MinMax>,
    },
    StatusDuration {
        #[serde(flatten)]
        status_filter: StatStatusFilter,
        #[serde(flatten)]
        skill_filter: StatSkillFilter,
    },
    StatusEscalation {
        #[serde(flatten)]
        status_filter: StatStatusFilter,
        #[serde(flatten)]
        skill_filter: StatSkillFilter,
    },
    StatusFaster {
        #[serde(flatten)]
        status_filter: StatStatusFilter,
        #[serde(flatten)]
        skill_filter: StatSkillFilter,
    },
    SuccessChance {
        #[serde(flatten)]
        skill_filter: StatSkillFilter,
        #[serde(default)]
        effect_type: Option<StatSkillEffectType>,
    },
    Speed(#[serde(default)] StatSkillFilter),
    RestoreOnHit {
        restore_type: RestoreType,
        #[serde(default)]
        skill_type: Option<SkillType>,
    },
    Restore {
        #[serde(default)]
        restore_type: Option<RestoreType>,
        #[serde(flatten)]
        skill_filter: StatSkillFilter,
    },
    Life,
    LifeRegen,
    Mana,
    ManaRegen,
    ManaCost {
        #[serde(flatten)]
        skill_filter: StatSkillFilter,
    },
    TakeFromManaBeforeLife,
    TakeFromLifeBeforeMana,
    MovementSpeed,
    ThreatGain,
    Lucky {
        #[serde(flatten)]
        skill_filter: StatSkillFilter,
        roll_type: LuckyRollType,
    },
    SkillConditionalModifier {
        stat: Box<StatType>,
        #[serde(flatten)]
        skill_filter: StatSkillFilter,
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
        #[serde(flatten)]
        skill_filter: StatSkillFilter,
        #[serde(default)]
        range: Option<SkillRange>,
        #[serde(default)]
        shape: Option<SkillShape>,
        #[serde(default)]
        repeat: Option<StatSkillRepeat>,
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
                    skill_filter,
                    damage_type,
                    min_max,
                    is_hit,
                },
                Damage {
                    skill_filter: skill_filter_2,
                    damage_type: damage_type_2,
                    min_max: min_max_2,
                    is_hit: is_hit_2,
                },
            ) => {
                skill_filter.is_match(skill_filter_2)
                    && compare_options(damage_type, damage_type_2)
                    && compare_options(min_max, min_max_2)
                    && compare_options(is_hit, is_hit_2)
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
                    skill_filter,
                    roll_type,
                },
                Lucky {
                    skill_filter: skill_filter_2,
                    roll_type: roll_type_2,
                },
            ) => skill_filter.is_match(skill_filter_2) && roll_type.is_match(roll_type_2),
            (
                SuccessChance {
                    skill_filter,
                    effect_type,
                },
                SuccessChance {
                    skill_filter: skill_filter_2,
                    effect_type: effect_type_2,
                },
            ) => {
                skill_filter.is_match(skill_filter_2) && compare_options(effect_type, effect_type_2)
            }
            (
                ManaCost { skill_filter },
                ManaCost {
                    skill_filter: skill_filter_2,
                },
            ) => skill_filter.is_match(skill_filter_2),
            (
                Restore {
                    restore_type,
                    skill_filter,
                },
                Restore {
                    restore_type: restore_type_2,
                    skill_filter: skill_filter_2,
                },
            ) => {
                compare_options(restore_type, restore_type_2)
                    && skill_filter.is_match(skill_filter_2)
            }
            (CritChance(first), CritChance(second))
            | (CritDamage(first), CritDamage(second))
            | (Speed(first), Speed(second)) => first.is_match(second),
            (Block(first), Block(second)) => compare_options(first, second),
            (Evade(first), Evade(second)) => compare_options(first, second),
            (
                StatusPower {
                    status_filter,
                    skill_filter,
                    min_max,
                },
                StatusPower {
                    status_filter: status_filter_2,
                    skill_filter: skill_filter_2,
                    min_max: min_max_2,
                },
            ) => {
                status_filter.is_match(status_filter_2)
                    && skill_filter.is_match(skill_filter_2)
                    && compare_options(min_max, min_max_2)
            }
            (
                StatusDuration {
                    status_filter,
                    skill_filter,
                },
                StatusDuration {
                    status_filter: status_filter_2,
                    skill_filter: skill_filter_2,
                },
            ) => status_filter.is_match(status_filter_2) && skill_filter.is_match(skill_filter_2),
            (
                StatusEscalation {
                    status_filter,
                    skill_filter,
                },
                StatusEscalation {
                    status_filter: status_filter_2,
                    skill_filter: skill_filter_2,
                },
            ) => status_filter.is_match(status_filter_2) && skill_filter.is_match(skill_filter_2),
            (
                StatusFaster {
                    status_filter,
                    skill_filter,
                },
                StatusFaster {
                    status_filter: status_filter_2,
                    skill_filter: skill_filter_2,
                },
            ) => status_filter.is_match(status_filter_2) && skill_filter.is_match(skill_filter_2),
            (
                StatusResistance {
                    status_id,
                    skill_type,
                },
                StatusResistance {
                    status_id: status_id_2,
                    skill_type: skill_type_2,
                },
            ) => {
                compare_options(status_id, status_id_2) && compare_options(skill_type, skill_type_2)
            }
            (SkillLevel(first), SkillLevel(second)) => first.is_match(second),
            (
                SkillConditionalModifier { skill_filter, .. },
                SkillConditionalModifier {
                    skill_filter: skill_filter_2,
                    ..
                },
            ) => skill_filter.is_match(skill_filter_2),
            _ => self == stat_type,
        }
    }
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
        matches!(
            (self, damage_type),
            (ArmorStatType::Physical, DamageType::Physical)
                | (
                    ArmorStatType::Fire | ArmorStatType::Elemental,
                    DamageType::Fire
                )
                | (
                    ArmorStatType::Poison | ArmorStatType::Elemental,
                    DamageType::Poison
                )
                | (
                    ArmorStatType::Storm | ArmorStatType::Elemental,
                    DamageType::Storm
                )
        )
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum StatSkillEffectType {
    FlatDamage {
        // damage_type: Option<DamageType>,
    },
    ApplyStatus {
        #[serde(default)]
        status_id: Option<StatusId>,
    },
    Restore {
        #[serde(default)]
        restore_type: Option<RestoreType>,
    },
    Resurrect,
    RefreshCooldown,
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
                ApplyStatus { status_id },
                ApplyStatus {
                    status_id: status_id_2,
                },
            ) => compare_options(status_id, status_id_2),
            _ => self == skill_effect_type,
        }
    }
}

impl From<&SkillEffectType> for Option<StatSkillEffectType> {
    fn from(value: &SkillEffectType) -> Self {
        match value {
            SkillEffectType::FlatDamage { .. } => Some(StatSkillEffectType::FlatDamage {}),
            SkillEffectType::ApplyStatus { status_id, .. } => {
                Some(StatSkillEffectType::ApplyStatus {
                    status_id: Some(status_id.clone()),
                })
            }
            SkillEffectType::Restore { restore_type, .. } => Some(StatSkillEffectType::Restore {
                restore_type: Some(*restore_type),
            }),
            SkillEffectType::Resurrect => Some(StatSkillEffectType::Resurrect),
            SkillEffectType::RefreshCooldown { .. } => Some(StatSkillEffectType::RefreshCooldown),
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
        value
            .into_iter()
            .fold(EffectsMap::default(), |mut effects_map, stat_effect| {
                effects_map.add_effect(stat_effect);
                effects_map
            })
    }
}

impl From<Vec<&StatEffect>> for EffectsMap {
    fn from(value: Vec<&StatEffect>) -> Self {
        value
            .iter()
            .fold(EffectsMap::default(), |mut effects_map, stat_effect| {
                effects_map.add_effect((*stat_effect).clone());
                effects_map
            })
    }
}

impl EffectsMap {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn add_effect(&mut self, stat_effect: StatEffect) {
        let StatEffect {
            stat,
            modifier,
            value,
            bypass_ignore,
        } = stat_effect;

        self.0
            .entry((stat, modifier, bypass_ignore))
            .and_modify(|entry| match modifier {
                Modifier::More => {
                    if *entry == 0.0 {
                        *entry = value;
                        return;
                    }

                    let sign = if *entry < 0.0 { -1.0 } else { 1.0 };

                    let factor = compute_more_factor(sign * value);
                    *entry = sign
                        * compute_more_factor(entry.abs() + factor + entry.abs() * factor * 0.01);
                }
                _ => *entry += value,
            })
            .or_insert(value);
    }

    pub fn combine_all(maps: impl Iterator<Item = EffectsMap>) -> Self {
        maps.flat_map(|m| m.0.into_iter()).fold(
            EffectsMap::default(),
            |mut result, ((stat, modifier, bypass_ignore), value)| {
                result.add_effect(StatEffect {
                    stat,
                    modifier,
                    value,
                    bypass_ignore,
                });
                result
            },
        )
    }

    pub fn iter(&self) -> impl Iterator<Item = StatEffect> + Clone {
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

impl Matchable for StatusId {
    fn is_match(&self, other: &Self) -> bool {
        self == other
    }
}

pub fn compare_options<T: Matchable>(first: &Option<T>, second: &Option<T>) -> bool {
    match (first, second) {
        (None, _) | (_, None) => true,
        (Some(a), Some(b)) => a.is_match(b),
    }
}
