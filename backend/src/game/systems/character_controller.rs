use rand::Rng;

use shared::data::{CharacterPrototype, CharacterState, SkillPrototype, SkillState};

use super::characters_updater;

pub fn use_skill(
    skill_prototype: &SkillPrototype,
    skill_state: &mut SkillState,
    target_state: &mut CharacterState,
    target_prototype: &CharacterPrototype,
) {
    let mut rng = rand::rng();

    skill_state.just_triggered = true;
    skill_state.is_ready = false;
    skill_state.elapsed_cooldown = 0.0;

    if skill_prototype.max_damages >= skill_prototype.min_damages {
        characters_updater::damage_character(
            rng.random_range(skill_prototype.min_damages..=skill_prototype.max_damages),
            target_state,
            target_prototype,
        );
    }
}

pub fn reset_character(character_state: &mut CharacterState) {
    character_state.just_hurt = false;
    for skill_sate in character_state.skill_states.iter_mut() {
        reset_skill(skill_sate)
    }
}

pub fn reset_skill(skill_state: &mut SkillState) {
    skill_state.just_triggered = false;
}
