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
    pub identifier: u64,

    pub name: String,
    pub portrait: String,

    pub max_health: u64, // TODO: change to big numbers num_bigint
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CharacterState {
    pub identifier: u64, // useful?
    pub is_alive: bool,
    pub health: u64, // TODO: change to big numbers num_bigint
}

impl CharacterState {
    pub fn init(prototype: &CharacterPrototype) -> Self {
        CharacterState {
            identifier: prototype.identifier,
            is_alive: true,
            health: prototype.max_health,
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

    pub max_mana: u64, // TODO: change to big numbers num_bigint
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct PlayerState {
    pub character_state: CharacterState,

    pub mana: u64, // TODO: change to big numbers num_bigint
}

impl PlayerState {
    pub fn init(prototype: &PlayerPrototype) -> Self {
        PlayerState {
            character_state: CharacterState::init(&prototype.character_prototype),
            mana: prototype.max_mana,
        }
    }
}
