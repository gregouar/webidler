use serde::{Deserialize, Serialize};

use super::{
    item_affix::{Modifier, StatType},
    skill::{DamageType, SkillEffect, SkillRange, SkillType},
};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventTrigger {
    OnHit(HitTrigger),
    OnTakeHit(HitTrigger),
    OnKill,
    OnWaveCompleted,
}

// TODO: replace by simple tag system?
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TriggeredEffect {
    pub trigger: EventTrigger,
    pub description: String,

    #[serde(flatten)]
    pub effect: TriggerEffectType,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TriggerEffectType {
    UseSkill, //TODO
    ApplySkillEffects {
        #[serde(default)]
        target: TriggerTarget,
        #[serde(default)]
        modifiers: Vec<TriggerEffectModifier>,
        effects: Vec<SkillEffect>,
    },
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
    AreaLevel,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Default)]
pub enum TriggerTarget {
    #[default]
    SameTarget,
    Source,
    // TODO: others?
}
