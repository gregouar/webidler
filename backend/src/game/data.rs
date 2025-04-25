use anyhow::Result;
use rand::Rng;
use std::path::PathBuf;

use serde::de::DeserializeOwned;
use tokio::fs;

use shared::data::{
    character::{CharacterSpecs, CharacterState},
    monster::{MonsterSpecs, MonsterState},
    player::{PlayerSpecs, PlayerState},
    skill::{SkillSpecs, SkillState},
    world::{WorldSpecs, WorldState},
};

pub async fn load_schema<S>(filepath: &PathBuf) -> Result<S>
where
    S: DeserializeOwned,
{
    Ok(serde_json::from_slice(
        &fs::read(&PathBuf::from("./data").join(filepath)).await?,
    )?)
}

pub trait DataInit<Specs> {
    fn init(specs: &Specs) -> Self;
}

impl DataInit<WorldSpecs> for WorldState {
    fn init(specs: &WorldSpecs) -> Self {
        WorldState {
            area_level: specs.starting_level,
            waves_done: 0,
        }
    }
}

impl DataInit<CharacterSpecs> for CharacterState {
    fn init(specs: &CharacterSpecs) -> Self {
        CharacterState {
            is_alive: true,
            health: specs.max_health,
            just_hurt: false,
            skill_states: specs
                .skill_specs
                .iter()
                .map(|p| SkillState::init(p))
                .collect(),
        }
    }
}

impl DataInit<PlayerSpecs> for PlayerState {
    fn init(specs: &PlayerSpecs) -> Self {
        PlayerState {
            character_state: CharacterState::init(&specs.character_specs),
            experience: 0.0,
            mana: specs.max_mana,
        }
    }
}

impl DataInit<MonsterSpecs> for MonsterState {
    fn init(specs: &MonsterSpecs) -> Self {
        let mut rng = rand::rng();
        MonsterState {
            character_state: CharacterState::init(&specs.character_specs),
            initiative: rng.random_range(0.0..specs.max_initiative),
        }
    }
}

impl DataInit<SkillSpecs> for SkillState {
    fn init(specs: &SkillSpecs) -> Self {
        let _ = specs;
        Self {
            elapsed_cooldown: 0.0,
            is_ready: false,
            just_triggered: false,
        }
    }
}
