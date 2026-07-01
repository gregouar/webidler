use std::{collections::HashMap, hash::Hash};

use serde::{Deserialize, Serialize};

use crate::data::{
    chance::BoundedChance,
    character_status::StatusId,
    conditional_modifier::{Condition, ConditionalModifier},
    modifier::ModifiableValue,
    skill::{DamageType, RepeatedSkillEffect, SkillType},
    stat_effect::StatEffect,
    trigger::TriggersMap,
    values::{AtLeastOne, BoundedValue, NonNegative, Percent},
};

use super::character_status::StatusMap;
pub use super::skill::{SkillSpecs, SkillState};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CharacterId {
    Player,
    Monster(usize),
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub enum CharacterSize {
    #[default]
    Small, // 1x1
    Large,      // 1x2
    Tall,       // 2x1
    Huge,       // 2x2
    Gargantuan, // 2x3
}

impl CharacterSize {
    pub fn get_xy_size(&self) -> (usize, usize) {
        match self {
            CharacterSize::Small => (1, 1),
            CharacterSize::Large => (2, 1),
            CharacterSize::Tall => (1, 2),
            CharacterSize::Huge => (2, 2),
            CharacterSize::Gargantuan => (3, 2),
        }
    }
}

// TODO: Split more for network usage? -> might become an hassle to handle in code...
// But I think I want it. We would have the "base specs (still updated by passives and skills)"
// and an "computed stats".
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CharacterSpecs {
    #[serde(flatten)]
    pub character_static: CharacterStatic,

    // TODO: All the above: move elsewhere ^^^
    // TODO: Should have CharacterComputed
    #[serde(flatten)]
    pub character_attrs: CharacterAttrs,
    #[serde(default)]
    pub skills_specs: Vec<SkillSpecs>,

    #[serde(default)]
    pub triggers: TriggersMap,
    #[serde(default)]
    pub effects: Vec<StatEffect>,

    #[serde(default, skip_serializing, skip_deserializing)]
    pub conditional_modifiers: Vec<ConditionalModifier>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CharacterStatic {
    pub name: String,
    pub portrait: String,

    #[serde(default)]
    pub size: CharacterSize,

    #[serde(default)]
    pub position_x: u8,
    #[serde(default)]
    pub position_y: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CharacterAttrs {
    pub max_life: ModifiableValue<AtLeastOne>,
    #[serde(default)]
    pub life_regen: ModifiableValue<f64>,

    #[serde(default)]
    pub max_mana: ModifiableValue<NonNegative>,
    #[serde(default)]
    pub mana_regen: ModifiableValue<f64>,

    #[serde(default)]
    pub take_from_mana_before_life: ModifiableValue<Percent>,
    #[serde(default)]
    pub take_from_life_before_mana: ModifiableValue<Percent>,

    #[serde(default)]
    pub armor: HashMap<DamageType, ModifiableValue<f64>>,

    #[serde(default)]
    pub block: HashMap<SkillType, BoundedChance>,
    #[serde(default)]
    pub block_damage: ModifiableValue<Percent>,

    #[serde(default)]
    pub evade: HashMap<DamageType, BoundedChance>,
    #[serde(default)]
    pub evade_damage: ModifiableValue<Percent>,

    #[serde(default)]
    pub status_resistances: HashMap<(SkillType, Option<StatusId>), ModifiableValue<f64>>,
    #[serde(default)]
    pub stun_lockout: ModifiableValue<NonNegative>,

    #[serde(default)]
    pub damage_resistance: HashMap<(SkillType, DamageType), ModifiableValue<BoundedValue<f64>>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CharacterState {
    pub life: NonNegative,
    pub mana: NonNegative,

    pub statuses: StatusMap,
    pub skills_states: Vec<SkillState>,

    pub is_alive: bool,
    pub just_hurt: bool,
    pub just_hurt_crit: bool,
    pub just_blocked: bool,
    pub just_evaded: bool,

    // This feels dirty
    #[serde(default, skip_serializing, skip_deserializing)]
    pub dirty_specs: bool,
    #[serde(default, skip_serializing, skip_deserializing)]
    pub monitored_conditions: HashMap<Condition, MonitoredCondition>,
    #[serde(default, skip_serializing, skip_deserializing)]
    pub repeated_skills: Vec<RepeatedSkillEffect>,
    #[serde(default, skip_serializing, skip_deserializing)]
    pub resurrected: bool,
}

// This shouldn't be here
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct MonitoredCondition {
    pub value: f64,
    pub duration: f64,
}

impl CharacterState {
    // TODO: Should we get rid of that?
    pub fn is_stunned(&self) -> bool {
        self.statuses
            .keys()
            .any(|status_id| status_id.as_str() == "stun")
    }
}
