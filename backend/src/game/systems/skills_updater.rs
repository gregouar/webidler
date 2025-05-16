use std::time::Duration;

use shared::data::skill::{SkillSpecs, SkillState};

pub fn update_skills_states(
    elapsed_time: Duration,
    skill_specs: &[SkillSpecs],
    skill_states: &mut [SkillState],
) {
    for (skill_specs, skill_state) in skill_specs.iter().zip(skill_states.iter_mut()) {
        skill_state.elapsed_cooldown += elapsed_time.as_secs_f32();
        if skill_state.elapsed_cooldown >= skill_specs.cooldown {
            skill_state.elapsed_cooldown = skill_specs.cooldown;
            skill_state.is_ready = true;
        } else {
            skill_state.is_ready = false;
        }
    }
}

pub fn reset_skills(skill_states: &mut Vec<SkillState>) {
    for skill_state in skill_states.iter_mut() {
        skill_state.just_triggered = false;
    }
}
