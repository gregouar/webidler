use serde::{Deserialize, Serialize};

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
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum SkillType {
    Attack,
    Spell,
    Weapon(ItemSlot),
}

impl Default for SkillType {
    fn default() -> Self {
        SkillType::Attack
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SkillEffect {
    #[serde(default)]
    pub range: Range,
    #[serde(default)]
    pub target_type: TargetType,
    #[serde(default)]
    pub shape: Shape,

    pub effect_type: SkillEffectType,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum TargetType {
    Enemy,
    Friend,
    Me,
}

impl Default for TargetType {
    fn default() -> Self {
        TargetType::Enemy
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum Range {
    Melee,
    Distance,
}

impl Default for Range {
    fn default() -> Self {
        Range::Melee
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum Shape {
    Single,
    Vertical2,
    Horizontal2,
    Horizontal3,
    Square4,
    All,
}

impl Default for Shape {
    fn default() -> Self {
        Shape::Single
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SkillEffectType {
    // TODO: merge multiple damage types à la weapon style
    FlatDamage {
        min: f64,
        max: f64,
        #[serde(default)]
        damage_type: DamageType,
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
            SkillEffectType::FlatDamage { min, max, .. } => {
                *min *= factor;
                *max *= factor;
            }
            SkillEffectType::Heal { min, max } => {
                *min *= factor;
                *max *= factor;
            }
        }
    }
}
