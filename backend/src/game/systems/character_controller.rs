use rand::Rng;

use shared::data::{CharacterSpecs, CharacterState, SkillSpecs, SkillState};

use super::characters_updater;

// TODO: What about multi-targets?

/// Return whether the target died from the attack
pub fn use_skill(
    skill_specs: &SkillSpecs,
    skill_state: &mut SkillState,
    target_state: &mut CharacterState,
    target_specs: &CharacterSpecs,
) -> bool {
    let mut rng = rand::rng();

    skill_state.just_triggered = true;
    skill_state.is_ready = false;
    skill_state.elapsed_cooldown = 0.0;

    if skill_specs.max_damages >= skill_specs.min_damages {
        return characters_updater::damage_character(
            rng.random_range(skill_specs.min_damages..=skill_specs.max_damages),
            target_state,
            target_specs,
        );
    }
    false
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
