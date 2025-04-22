// TODO: split in multiple files

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HelloSchema {
    pub greeting: String,
    pub value: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OtherSchema {
    pub other: String,
    pub value: i32,
}

// World

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct WorldSpecs {
    pub name: String,
    pub starting_level: u16,
    pub musics: Vec<String>,
    pub header_background: String,
    pub footer_background: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct WorldState {
    pub area_level: u16,
}

// Character
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CharacterSpecs {
    pub name: String,
    pub portrait: String,

    pub max_health: f64,
    pub health_regen: f64,

    pub skill_specs: Vec<SkillSpecs>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CharacterState {
    pub is_alive: bool,
    pub health: f64,
    pub just_hurt: bool,

    pub skill_states: Vec<SkillState>,
}

// Monster

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct MonsterSpecs {
    pub character_specs: CharacterSpecs,

    pub max_initiative: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct MonsterState {
    pub character_state: CharacterState,

    pub initiative: f32,
}

// Player

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct PlayerSpecs {
    pub character_specs: CharacterSpecs,

    pub max_mana: f64,
    pub mana_regen: f64,

    pub auto_skills: Vec<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct PlayerState {
    pub character_state: CharacterState,

    pub mana: f64,
}

// Skill

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SkillSpecs {
    pub name: String,
    pub icon: String,

    pub cooldown: f32,
    pub mana_cost: f64,

    pub min_damages: f64, // Could this be a range?
    pub max_damages: f64,
    // TODO: target type, target amount, range etc
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SkillState {
    pub elapsed_cooldown: f32,
    pub is_ready: bool,
    pub just_triggered: bool,
}
