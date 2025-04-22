use std::time::Duration;

use shared::data::{CharacterSpecs, CharacterState};

pub fn update_character_state(
    elapsed_time: Duration,
    specs: &CharacterSpecs,
    state: &mut CharacterState,
) {
    if !state.is_alive {
        return;
    }

    state.health = specs
        .max_health
        .min(state.health + (elapsed_time.as_secs_f64() * specs.health_regen));

    for (skill_specs, skill_state) in specs.skill_specs.iter().zip(state.skill_states.iter_mut()) {
        skill_state.elapsed_cooldown += elapsed_time.as_secs_f32();
        if skill_state.elapsed_cooldown >= skill_specs.cooldown {
            skill_state.elapsed_cooldown = skill_specs.cooldown;
            skill_state.is_ready = true;
        } else {
            skill_state.is_ready = false;
        }
    }
}

pub fn damage_character(
    damages: f64,
    target_state: &mut CharacterState,
    target_specs: &CharacterSpecs,
) {
    target_state.health = (target_state.health - damages)
        .max(0.0)
        .min(target_specs.max_health);

    if damages > 0.0 {
        target_state.just_hurt = true;
    }
    if target_state.health == 0.0 {
        target_state.is_alive = false;
    }
}
