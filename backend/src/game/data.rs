use anyhow::{Context, Result};
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

use crate::game::utils::rng;

pub async fn load_json<S>(filepath: &PathBuf) -> Result<S>
where
    S: DeserializeOwned,
{
    let file_path = PathBuf::from("./data").join(filepath);
    Ok(serde_json::from_slice(
        &fs::read(&file_path)
            .await
            .with_context(|| format!("Failed to read file: {:?}", file_path))?,
    )
    .with_context(|| format!("Failed to parse json from: {:?}", file_path))?)
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
            just_died: false,
            health: specs.max_health,
            just_hurt: false,
        }
    }
}

impl DataInit<PlayerSpecs> for PlayerState {
    fn init(specs: &PlayerSpecs) -> Self {
        PlayerState {
            character_state: CharacterState::init(&specs.character_specs),
            skills_states: specs
                .skills_specs
                .iter()
                .map(|p| SkillState::init(p))
                .collect(),
            mana: specs.max_mana,
            just_leveled_up: false,
        }
    }
}

impl DataInit<MonsterSpecs> for MonsterState {
    fn init(specs: &MonsterSpecs) -> Self {
        MonsterState {
            character_state: CharacterState::init(&specs.character_specs),
            skill_states: specs
                .skill_specs
                .iter()
                .map(|p| SkillState::init(p))
                .collect(),
            initiative: rng::random_range(0.0..specs.max_initiative).unwrap_or_default(),
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
