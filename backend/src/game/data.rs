use rand::Rng;

use shared::data::{
    CharacterPrototype, CharacterState, MonsterPrototype, MonsterState, PlayerPrototype,
    PlayerState, SkillPrototype, SkillState,
};

pub trait DataInit<Prototype> {
    fn init(prototype: &Prototype) -> Self;
}

impl DataInit<CharacterPrototype> for CharacterState {
    fn init(prototype: &CharacterPrototype) -> Self {
        CharacterState {
            // identifier: prototype.identifier,
            is_alive: true,
            health: prototype.max_health,
            just_hurt: false,
            skill_states: prototype
                .skill_prototypes
                .iter()
                .map(|p| SkillState::init(p))
                .collect(),
        }
    }
}

impl DataInit<PlayerPrototype> for PlayerState {
    fn init(prototype: &PlayerPrototype) -> Self {
        PlayerState {
            character_state: CharacterState::init(&prototype.character_prototype),
            mana: prototype.max_mana,
        }
    }
}

impl DataInit<MonsterPrototype> for MonsterState {
    fn init(prototype: &MonsterPrototype) -> Self {
        let mut rng = rand::rng();
        MonsterState {
            character_state: CharacterState::init(&prototype.character_prototype),
            initiative: rng.random_range(0.0..prototype.max_initiative),
        }
    }
}

impl DataInit<SkillPrototype> for SkillState {
    fn init(prototype: &SkillPrototype) -> Self {
        let _ = prototype;
        Self {
            elapsed_cooldown: 0.0,
            is_ready: false,
            just_triggered: false,
        }
    }
}
