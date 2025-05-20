use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub use super::effect::DamageType;
use super::item::ItemSlot;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SkillState {
    pub elapsed_cooldown: f32,
    pub is_ready: bool,
    pub just_triggered: bool,
}

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

    pub effects: Vec<SkillEffect>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SkillSpecs {
    pub base: BaseSkillSpecs,

    pub cooldown: f32,
    pub mana_cost: f64,

    pub upgrade_level: u16,
    pub next_upgrade_cost: f64,

    pub effects: Vec<SkillEffect>,

    pub item_slot: Option<ItemSlot>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Default)]
pub enum SkillType {
    #[default]
    Attack,
    Spell,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SkillEffect {
    #[serde(default)]
    pub range: Range,
    #[serde(default)]
    pub target_type: TargetType,
    #[serde(default)]
    pub shape: Shape,

    #[serde(flatten)]
    pub effect_type: SkillEffectType,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default)]
pub enum TargetType {
    #[default]
    Enemy,
    Friend,
    Me,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Default)]
pub enum Range {
    #[default]
    Melee,
    Distance,
    Any,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Default)]
pub enum Shape {
    #[default]
    Single,
    Vertical2,
    Horizontal2,
    Horizontal3,
    Square4,
    All,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SkillEffectType {
    FlatDamage {
        damage: HashMap<DamageType, (f64, f64)>,
        #[serde(default)]
        crit_chances: f32,
        #[serde(default)]
        crit_damage: f64,
    },
    Heal {
        min: f64,
        max: f64,
    },
}

impl SkillEffect {
    pub fn increase_effect(&mut self, factor: f64) {
        match &mut self.effect_type {
            SkillEffectType::FlatDamage { damage, .. } => {
                for (min, max) in damage.values_mut() {
                    *min *= factor;
                    *max *= factor;
                }
            }
            SkillEffectType::Heal { min, max } => {
                *min *= factor;
                *max *= factor;
            }
        }
    }
}
