use serde::{Deserialize, Serialize};

use crate::data::{
    character::CharacterId, item::SkillShape, skill::TargetType, stat_effect::StatStatusType,
};

use super::{
    skill::{DamageType, SkillEffect, SkillRange, SkillType},
    stat_effect::{Modifier, StatType},
};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventTrigger {
    OnHit(HitTrigger),
    OnTakeHit(HitTrigger),
    OnKill(KillTrigger),
    OnWaveCompleted,
    OnThreatIncreased,
    OnDeath(TargetType),
}

// TODO: replace by simple tag system?
#[derive(
    Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, Default, PartialOrd, Ord,
)]
pub struct HitTrigger {
    #[serde(default)]
    pub skill_type: Option<SkillType>,
    #[serde(default)]
    pub range: Option<SkillRange>,
    #[serde(default)]
    pub is_crit: Option<bool>,
    #[serde(default)]
    pub is_blocked: Option<bool>,
    #[serde(default)]
    pub is_hurt: Option<bool>,
    #[serde(default)]
    pub is_triggered: Option<bool>,
    // TODO: Track skill id?
}

#[derive(
    Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, Default, PartialOrd, Ord,
)]
pub struct KillTrigger {
    #[serde(default)]
    pub is_stunned: Option<bool>,
    #[serde(default)]
    pub is_debuffed: Option<bool>,
    #[serde(default)]
    pub is_damaged_over_time: Option<DamageType>,
    // TODO: more?
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct TriggerSpecs {
    pub trigger_id: String,
    #[serde(default)]
    pub description: String,
    #[serde(flatten)]
    pub triggered_effect: TriggeredEffect,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct TriggeredEffect {
    #[serde(flatten)]
    pub trigger: EventTrigger,
    #[serde(default)]
    pub target: TriggerTarget,
    #[serde(default)]
    pub modifiers: Vec<TriggerEffectModifier>,

    #[serde(default)]
    pub skill_range: SkillRange,
    #[serde(default)]
    pub skill_type: SkillType,
    pub effects: Vec<SkillEffect>,

    #[serde(default)]
    pub skill_shape: SkillShape,

    #[serde(default)] // For retro compatibility
    pub owner: Option<CharacterId>,
    #[serde(default)] // For retro compatibility
    pub inherit_modifiers: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct TriggerEffectModifier {
    pub stat: StatType,
    pub modifier: Modifier,
    pub factor: f64,
    pub source: TriggerEffectModifierSource,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum TriggerEffectModifierSource {
    HitDamage(Option<DamageType>),
    HitCrit,
    AreaLevel,
    StatusValue(Option<StatStatusType>),
    StatusDuration(Option<StatStatusType>),
    StatusStacks(Option<StatStatusType>),
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Default)]
pub enum TriggerTarget {
    #[default]
    SameTarget,
    Source,
    Me,
    // TODO: others?
}
