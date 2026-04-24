use shared::data::{
    area::{AreaSpecs, AreaState},
    chance::ChanceRange,
    character::CharacterState,
    character_status::StatusMap,
    monster::{MonsterSpecs, MonsterState},
    passive::{PassivesTreeAscension, PassivesTreeState},
    player::{CharacterSpecs, PlayerBaseSpecs, PlayerSpecs, PlayerState},
};

use crate::game::utils::rng::Rollable;
pub trait DataInit<Specs> {
    fn init(specs: Specs) -> Self;
}

impl DataInit<&AreaSpecs> for AreaState {
    fn init(_: &AreaSpecs) -> Self {
        AreaState {
            area_level: 1,
            is_boss: false,
            waves_done: 1,
            max_area_level: 0,
            max_area_level_ever: 0,
            last_champion_spawn: 0,
            auto_progress: true,
            going_back: 0,
            rush_mode: false,
        }
    }
}

impl DataInit<&CharacterSpecs> for CharacterState {
    fn init(specs: &CharacterSpecs) -> Self {
        CharacterState {
            life: specs.character_attrs.max_life.get().into(),
            mana: specs.character_attrs.max_mana.get().into(),

            statuses: StatusMap::default(),
            skills_states: specs
                .skills_specs
                .iter()
                .map(|_| Default::default())
                .collect(),

            is_alive: true,
            just_hurt: false,
            just_hurt_crit: false,
            just_blocked: false,
            just_evaded: false,

            dirty_specs: true,
            monitored_conditions: Default::default(),
            repeated_skills: Default::default(),
        }
    }
}

impl DataInit<&PlayerBaseSpecs> for PlayerSpecs {
    fn init(specs: &PlayerBaseSpecs) -> Self {
        PlayerSpecs {
            character_specs: CharacterSpecs {
                character_static: specs.character_static.clone(),
                character_attrs: specs.character_attrs.clone(),
                skills_specs: Default::default(),
                triggers: Default::default(),
                effects: specs.effects.clone(),
                conditional_modifiers: Default::default(),
            },
            movement_cooldown: specs.movement_cooldown.clone(),
            gold_find: specs.gold_find.clone(),
            threat_gain: specs.threat_gain.clone(),
        }
    }
}

impl DataInit<&PlayerSpecs> for PlayerState {
    fn init(specs: &PlayerSpecs) -> Self {
        PlayerState {
            character_state: CharacterState::init(&specs.character_specs),
        }
    }
}

impl DataInit<&MonsterSpecs> for MonsterState {
    fn init(specs: &MonsterSpecs) -> Self {
        let mut monster_state = MonsterState {
            character_state: CharacterState::init(&specs.character_specs),
            gold_reward: 0.0,
            gems_reward: 0.0,
        };

        for skill_state in monster_state.character_state.skills_states.iter_mut() {
            skill_state.elapsed_cooldown = ChanceRange {
                min: 0.0,
                max: 1.0,
                ..Default::default()
            }
            .roll()
            .into();
        }

        monster_state
    }
}

// impl DataInit<BaseSkillSpecs> for SkillSpecs {
//     fn init(specs: BaseSkillSpecs) -> Self {
//         Self {
//             cooldown: specs.cooldown.into(),
//             mana_cost: specs.mana_cost.into(),
//             upgrade_level: 1,
//             next_upgrade_cost: specs.upgrade_cost,
//             targets: specs.targets.clone(),
//             triggers: specs.triggers.clone(),
//             item_slot: None,
//             level_modifier: 0,
//         }
//     }
// }

// impl DataInit<&SkillSpecs> for SkillState {
//     fn init(specs: &SkillSpecs) -> Self {
//         let _ = specs;
//         Self {
//             elapsed_cooldown: Default::default(),
//             is_ready: false,
//             just_triggered: false,
//         }
//     }
// }

impl DataInit<PassivesTreeAscension> for PassivesTreeState {
    fn init(ascension: PassivesTreeAscension) -> Self {
        Self {
            purchased_nodes: Default::default(),
            ascension,
        }
    }
}
