use serde::{Deserialize, Serialize};

use crate::data::trigger::TriggerSpecs;

pub use super::stat_effect::DamageType;
use super::{
    character_status::StatusType, item::ItemSlot, passive::StatEffect, stat_effect::DamageMap,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BaseSkillSpecs {
    pub name: String,
    pub icon: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub skill_type: SkillType,

    pub cooldown: f32,
    #[serde(default)]
    pub mana_cost: f64,

    #[serde(default)]
    pub upgrade_cost: f64,
    #[serde(default)]
    pub upgrade_effects: Vec<StatEffect>,

    #[serde(default)]
    pub targets: Vec<SkillTargetsGroup>,
    #[serde(default)]
    pub triggers: Vec<TriggerSpecs>,
    // TODO: special upgrades at some levels?
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SkillSpecs {
    pub base: BaseSkillSpecs,

    pub cooldown: f32,
    pub mana_cost: f64,

    pub upgrade_level: u16,
    pub next_upgrade_cost: f64,

    pub targets: Vec<SkillTargetsGroup>,
    pub triggers: Vec<TriggerSpecs>,

    pub item_slot: Option<ItemSlot>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SkillState {
    pub elapsed_cooldown: f32,

    pub is_ready: bool,
    pub just_triggered: bool,
}

#[derive(
    Serialize, Deserialize, Debug, Clone, Copy, Default, PartialEq, Eq, Hash, PartialOrd, Ord,
)]
pub enum SkillType {
    #[default]
    Attack,
    Spell,
}

impl SkillType {
    pub fn iter() -> impl Iterator<Item = SkillType> {
        [SkillType::Attack, SkillType::Spell].into_iter()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SkillTargetsGroup {
    #[serde(default)]
    pub range: SkillRange,
    #[serde(default)]
    pub target_type: TargetType,
    #[serde(default)]
    pub shape: SkillShape,
    #[serde(default)]
    pub target_dead: bool,

    pub effects: Vec<SkillEffect>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SkillEffect {
    #[serde(default)]
    pub failure_chances: f32,

    #[serde(flatten)]
    pub effect_type: SkillEffectType,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum SkillEffectType {
    FlatDamage {
        damage: DamageMap,
        #[serde(default)]
        crit_chances: f32,
        #[serde(default)]
        crit_damage: f64,
    },
    ApplyStatus {
        statuses: Vec<ApplyStatusEffect>,
        min_duration: f64,
        max_duration: f64,
    },
    Restore {
        restore_type: RestoreType,
        min: f64,
        max: f64,
    },
    Resurrect,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ApplyStatusEffect {
    pub status_type: StatusType,
    pub min_value: f64,
    pub max_value: f64,
    #[serde(default)]
    pub cumulate: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default, PartialEq)]
pub enum TargetType {
    #[default]
    Enemy,
    Friend,
    Me,
}

#[derive(
    Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Default, Eq, Hash, PartialOrd, Ord,
)]
pub enum SkillRange {
    #[default]
    Melee,
    Distance,
    Any,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Default)]
pub enum SkillShape {
    #[default]
    Single,
    Vertical2,
    Horizontal2,
    Horizontal3,
    Square4,
    All,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum RestoreType {
    Life,
    Mana,
}
