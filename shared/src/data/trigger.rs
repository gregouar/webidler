use serde::{Deserialize, Serialize};

use super::skill::{SkillEffect, SkillRange, SkillType};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventTrigger {
    OnHit(HitTrigger),
    OnCriticalHit(HitTrigger),
    OnTakeHit(HitTrigger),
    OnBlock(HitTrigger),
    OnKill,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HitTrigger {
    #[serde(default)]
    pub skill_type: Option<SkillType>,
    #[serde(default)]
    pub range: Option<SkillRange>,
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
        effects: Vec<SkillEffect>,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Default)]
pub enum TriggerTarget {
    #[default]
    SameTarget,
    Source,
    // TODO: others?
}
