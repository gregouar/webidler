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

// Character

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CharacterPrototype {
    // pub identifier: u64,
    pub name: String,
    pub portrait: String,

    // TODO: This will not work well with temporary buffs etc... ?
    pub max_health: f64,
    pub health_regen: f64,

    pub skill_prototypes: Vec<SkillPrototype>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CharacterState {
    // pub identifier: u64, // useful?
    pub is_alive: bool,
    pub health: f64,

    // pub initiative_cooldown: f64, // TODO
    pub skill_states: Vec<SkillState>,
}

impl CharacterState {
    pub fn init(prototype: &CharacterPrototype) -> Self {
        CharacterState {
            // identifier: prototype.identifier,
            is_alive: true,
            health: prototype.max_health,
            skill_states: prototype
                .skill_prototypes
                .iter()
                .map(|p| SkillState::init(p))
                .collect(),
        }
    }
}

// Monster

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct MonsterPrototype {
    pub character_prototype: CharacterPrototype,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct MonsterState {
    pub character_state: CharacterState,
}

impl MonsterState {
    pub fn init(prototype: &MonsterPrototype) -> Self {
        MonsterState {
            character_state: CharacterState::init(&prototype.character_prototype),
        }
    }
}

// Player

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct PlayerPrototype {
    pub character_prototype: CharacterPrototype,

    pub max_mana: f64,
    pub mana_regen: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct PlayerState {
    pub character_state: CharacterState,

    pub mana: f64,
}

impl PlayerState {
    pub fn init(prototype: &PlayerPrototype) -> Self {
        PlayerState {
            character_state: CharacterState::init(&prototype.character_prototype),
            mana: prototype.max_mana,
        }
    }
}

// Skill

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SkillPrototype {
    pub name: String,
    pub icon: String, // TODO better...

    pub cooldown: f32,
    pub mana_cost: f64,

    pub min_damages: f64, // Could this be a range?
    pub max_damages: f64,
    // TODO: have proper skill data structure...
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SkillState {
    pub elapsed_cooldown: f32,
    pub is_ready: bool,
    pub just_triggered: bool,
}

impl SkillState {
    pub fn init(_prototype: &SkillPrototype) -> Self {
        Self {
            elapsed_cooldown: 0.0,
            is_ready: false,
            just_triggered: false,
        }
    }
}
