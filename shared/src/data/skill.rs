use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SkillSpecs {
    pub name: String,
    pub icon: String,
    #[serde(default)]
    pub description: String,

    pub cooldown: f32,
    #[serde(default)]
    pub mana_cost: f64,

    // TODO: all the following should be per SkillEffect!
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
    // TODO: move to specs...
    pub next_upgrade_cost: f64,
    pub upgrade_level: u16,

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
