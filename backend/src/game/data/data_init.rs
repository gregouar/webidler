use std::collections::HashSet;

use shared::{
    constants::{DEFAULT_MAX_LEVEL, SKILL_BASE_COST},
    data::{
        area::{AreaSpecs, AreaState},
        character::{CharacterSpecs, CharacterState},
        character_status::StatusMap,
        monster::{MonsterSpecs, MonsterState},
        passive::{PassivesTreeAscension, PassivesTreeState},
        player::{PlayerSpecs, PlayerState},
        skill::{BaseSkillSpecs, SkillSpecs, SkillState},
        stat_effect::EffectsMap,
    },
};

use crate::game::utils::rng::Rollable;
pub trait DataInit<Specs> {
    fn init(specs: Specs) -> Self;
}

impl DataInit<&AreaSpecs> for AreaState {
    fn init(specs: &AreaSpecs) -> Self {
        AreaState {
            area_level: specs.starting_level,
            is_boss: false,
            waves_done: 1,
            max_area_level: 0,
            max_area_level_ever: 0,
            last_champion_spawn: 0,
            auto_progress: true,
            going_back: 0,
            end_quest: false,
        }
    }
}

impl DataInit<&CharacterSpecs> for CharacterState {
    fn init(specs: &CharacterSpecs) -> Self {
        CharacterState {
            life: specs.max_life,
            mana: specs.max_mana,

            statuses: StatusMap::default(),
            dirty_specs: true,

            is_alive: true,
            just_hurt: false,
            just_hurt_crit: false,
            just_blocked: false,
        }
    }
}

impl DataInit<CharacterSpecs> for PlayerSpecs {
    fn init(specs: CharacterSpecs) -> Self {
        PlayerSpecs {
            max_area_level: 0,
            character_specs: specs.clone(),
            skills_specs: vec![],
            auto_skills: vec![],
            max_skills: 4,
            buy_skill_cost: SKILL_BASE_COST,
            bought_skills: HashSet::new(),
            level: 1,
            experience_needed: 20.0,
            movement_cooldown: 3.0,
            gold_find: 100.0,
            threat_gain: 100.0,
            effects: EffectsMap::default(),
            max_level: DEFAULT_MAX_LEVEL,
        }
    }
}

impl DataInit<&PlayerSpecs> for PlayerState {
    fn init(specs: &PlayerSpecs) -> Self {
        PlayerState {
            character_state: CharacterState::init(&specs.character_specs),
            skills_states: specs.skills_specs.iter().map(SkillState::init).collect(),
        }
    }
}

impl DataInit<&MonsterSpecs> for MonsterState {
    fn init(specs: &MonsterSpecs) -> Self {
        MonsterState {
            character_state: CharacterState::init(&specs.character_specs),
            skill_states: specs.skill_specs.iter().map(SkillState::init).collect(),
            initiative: specs.initiative.roll(),
            gold_reward: 0.0,
            gems_reward: 0.0,
        }
    }
}

impl DataInit<BaseSkillSpecs> for SkillSpecs {
    fn init(specs: BaseSkillSpecs) -> Self {
        Self {
            cooldown: specs.cooldown,
            mana_cost: specs.mana_cost,
            upgrade_level: 1,
            next_upgrade_cost: specs.upgrade_cost,
            targets: specs.targets.clone(),
            triggers: specs.triggers.clone(),
            item_slot: None,
            base: specs,
            level_modifier: 0,
        }
    }
}

impl DataInit<&SkillSpecs> for SkillState {
    fn init(specs: &SkillSpecs) -> Self {
        let _ = specs;
        Self {
            elapsed_cooldown: 0.0,
            is_ready: false,
            just_triggered: false,
        }
    }
}

impl DataInit<PassivesTreeAscension> for PassivesTreeState {
    fn init(ascension: PassivesTreeAscension) -> Self {
        Self {
            purchased_nodes: Default::default(),
            ascension,
        }
    }
}
