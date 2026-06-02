use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::data::{
    character::CharacterId, character_status::StatusId, conditional_modifier::Condition,
    item::SkillShape, modifier::Modifier, skill::TargetType, stat_effect::StatStatusFilter,
};

use super::{
    skill::{DamageType, SkillEffect, SkillRange, SkillType},
    stat_effect::StatType,
};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct TriggersMap(HashMap<EventTrigger, Vec<OwnedTrigger>>);

impl TriggersMap {
    pub fn new() -> Self {
        Self(Default::default())
    }

    pub fn push(
        &mut self,
        trigger: EventTrigger,
        trigger_effect: TriggerEffect,
        owner: Option<CharacterId>,
    ) {
        self.0.entry(trigger).or_default().push(OwnedTrigger {
            trigger_effect,
            owner,
        })
    }

    pub fn iter(&self) -> impl Iterator<Item = (&EventTrigger, &Vec<OwnedTrigger>)> {
        self.0.iter()
    }

    pub fn into_iter(self) -> impl Iterator<Item = (EventTrigger, Vec<OwnedTrigger>)> {
        self.0.into_iter()
    }

    pub fn effects_iter_mut(&mut self) -> impl Iterator<Item = &mut TriggerEffect> {
        self.0.values_mut().flat_map(|owned_effects| {
            owned_effects
                .iter_mut()
                .map(|owned_effect| &mut owned_effect.trigger_effect)
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum EventTrigger {
    OnHit(HitTrigger),
    OnTakeHit(HitTrigger),
    OnKill(KillTrigger),
    OnWaveCompleted,
    OnThreatIncreased,
    OnDeath(TargetType),
    OnApplyStatus(StatusTrigger),
    OnReceiveStatus(StatusTrigger),
}

// TODO: replace by simple tag system?
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
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
    #[serde(default)]
    pub damage_type: Option<DamageType>,
    #[serde(default)]
    pub skill_ids: Option<Vec<String>>,
    #[serde(default)]
    pub conditions: Vec<Condition>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
pub struct StatusTrigger {
    #[serde(default)]
    pub skill_type: Option<SkillType>,
    #[serde(flatten)]
    pub status_filter: StatStatusFilter,
    #[serde(default)]
    pub is_triggered: Option<bool>,
    #[serde(default)]
    pub is_evaded: Option<bool>,
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
pub struct KillTrigger {
    #[serde(default)]
    pub conditions: Vec<Condition>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct TriggerSpecs {
    #[serde(flatten)]
    pub trigger: EventTrigger,
    #[serde(default)]
    pub description: Option<String>, // TODO: Do something else?

    #[serde(flatten)]
    pub trigger_effect: TriggerEffect,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct TriggerEffect {
    pub trigger_id: String,
    #[serde(default)]
    pub target: TriggerTarget,
    #[serde(default)]
    pub modifiers: Vec<TriggerEffectModifier>,
    #[serde(default)]
    pub trigger_propagate: bool, // If true, will reset trigger depth

    pub skill_type: SkillType,
    #[serde(default)]
    pub skill_range: SkillRange,
    #[serde(default)]
    pub skill_shape: SkillShape,

    pub effects: Vec<SkillEffect>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct TriggerEffectModifier {
    pub stat: StatType,
    pub modifier: Modifier,
    pub factor: f64,
    pub source: TriggerEffectModifierSource,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum TriggerEffectModifierSource {
    HitDamage(Option<DamageType>),
    AreaLevel,
    StatusValue {
        #[serde(default)]
        status_id: Option<StatusId>,
        #[serde(default)]
        skill_type: Option<SkillType>,
    },
    StatusDuration {
        #[serde(default)]
        status_id: Option<StatusId>,
        #[serde(default)]
        skill_type: Option<SkillType>,
    },
    StatusStacks {
        #[serde(default)]
        status_id: Option<StatusId>,
        #[serde(default)]
        skill_type: Option<SkillType>,
    },
    TriggerStatusValue,
    // TODO: Move to conditional modifiers?
    HitCrit,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Default)]
pub enum TriggerTarget {
    #[default]
    SameTarget,
    Source,
    Me,
    // TODO: others?
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct OwnedTrigger {
    pub trigger_effect: TriggerEffect,
    pub owner: Option<CharacterId>,
}
