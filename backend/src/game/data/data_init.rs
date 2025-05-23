use std::collections::HashMap;

use shared::data::{
    character::{CharacterSpecs, CharacterState},
    monster::{MonsterSpecs, MonsterState},
    player::{PlayerSpecs, PlayerState},
    skill::{BaseSkillSpecs, SkillSpecs, SkillState},
    world::{WorldSpecs, WorldState},
};

use crate::game::utils::rng;
pub trait DataInit<Specs> {
    fn init(specs: &Specs) -> Self;
}

impl DataInit<WorldSpecs> for WorldState {
    fn init(specs: &WorldSpecs) -> Self {
        WorldState {
            area_level: specs.starting_level,
            waves_done: 0,
            auto_progress: true,
            going_back: 0,
        }
    }
}

impl DataInit<CharacterSpecs> for CharacterState {
    fn init(specs: &CharacterSpecs) -> Self {
        CharacterState {
            is_alive: true,
            health: specs.max_life,
            statuses: HashMap::new(),

            just_died: false,
            just_hurt: false,
            just_hurt_crit: false,
            just_blocked: false,
        }
    }
}

impl DataInit<PlayerSpecs> for PlayerState {
    fn init(specs: &PlayerSpecs) -> Self {
        PlayerState {
            character_state: CharacterState::init(&specs.character_specs),
            skills_states: specs.skills_specs.iter().map(SkillState::init).collect(),
            mana: specs.max_mana,
            just_leveled_up: false,
        }
    }
}

impl DataInit<MonsterSpecs> for MonsterState {
    fn init(specs: &MonsterSpecs) -> Self {
        MonsterState {
            character_state: CharacterState::init(&specs.character_specs),
            skill_states: specs.skill_specs.iter().map(SkillState::init).collect(),
            initiative: rng::random_range(0.0..specs.max_initiative).unwrap_or_default(),
        }
    }
}

impl DataInit<BaseSkillSpecs> for SkillSpecs {
    fn init(specs: &BaseSkillSpecs) -> Self {
        Self {
            base: specs.clone(),
            cooldown: specs.cooldown,
            mana_cost: specs.mana_cost,
            upgrade_level: 1,
            next_upgrade_cost: specs.upgrade_cost,
            targets: specs.targets.clone(),
            item_slot: None,
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
