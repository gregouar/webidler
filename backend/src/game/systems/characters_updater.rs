use std::time::Duration;

use shared::data::{CharacterPrototype, CharacterState};

pub fn update_character_state(
    elapsed_time: Duration,
    prototype: &CharacterPrototype,
    state: &mut CharacterState,
) {
    if !state.is_alive {
        return;
    }

    state.health = prototype
        .max_health
        .min(state.health + (elapsed_time.as_secs_f64() * prototype.health_regen));

    for (skill_prototype, skill_state) in prototype
        .skill_prototypes
        .iter()
        .zip(state.skill_states.iter_mut())
    {
        skill_state.elapsed_cooldown += elapsed_time.as_secs_f32();
        if skill_state.elapsed_cooldown >= skill_prototype.cooldown {
            skill_state.elapsed_cooldown = skill_prototype.cooldown;
            skill_state.is_ready = true;
        } else {
            skill_state.is_ready = false;
        }
    }
}

pub fn damage_character(
    damages: f64,
    target_state: &mut CharacterState,
    target_prototype: &CharacterPrototype,
) {
    target_state.health = (target_state.health - damages)
        .max(0.0)
        .min(target_prototype.max_health);

    if damages > 0.0 {
        target_state.just_hurt = true;
    }
    if target_state.health == 0.0 {
        target_state.is_alive = false;
    }
}
