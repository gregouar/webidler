use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SkillSpecs {
    pub name: String,
    pub icon: String,
    pub description: String,

    pub cooldown: f32,
    #[serde(default)]
    pub mana_cost: f64,

    #[serde(default)]
    pub range: Range,
    #[serde(default)]
    pub target_type: TargetType,
    #[serde(default)]
    pub shape: Shape,

    pub min_damages: f64,
    pub max_damages: f64,
    // TODO: Damage type, dot, all kind of other buffs and effects
    // -> not sure yet what is the best approach there, maybe should go for some kind
    // of vector of enum? Need way to compose effects etc probably
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SkillState {
    // TODO: Upgrade system
    // pub next_upgrade_cost: f64,
    // pub upgrade_level: u16,

    // pub min_damages: f64,
    // pub max_damages: f64,
    pub elapsed_cooldown: f32,
    pub is_ready: bool,
    pub just_triggered: bool,
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

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum Range {
    Front,
    Middle,
    Back,
}

impl Default for Range {
    fn default() -> Self {
        Range::Front
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
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
