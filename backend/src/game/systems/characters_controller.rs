use std::collections::HashMap;

use shared::data::{
    character::{CharacterId, CharacterSpecs, CharacterState},
    character_status::{StatusState, StatusType},
    skill::{DamageType, SkillType},
};

use crate::game::{
    data::event::{DamageEvent, EventsQueue, GameEvent},
    utils::{increase_factors, rng},
};

pub type Target<'a> = (CharacterId, (&'a CharacterSpecs, &'a mut CharacterState));

pub fn attack_character(
    events_queue: &mut EventsQueue,
    target: &mut Target,
    attacker: CharacterId,
    damage: HashMap<DamageType, f64>,
    skill_type: SkillType,
    is_crit: bool,
) {
    let (target_id, (target_specs, target_state)) = target;

    let amount: f64 = damage
        .iter()
        .map(|(damage_type, amount)| {
            decrease_damage_from_armor(*amount, *damage_type, target_specs)
        })
        .sum();

    let damage_event = DamageEvent {
        attacker,
        target: *target_id,
        skill_type,
        damage,
    };

    let is_blocked = skill_type == SkillType::Attack
        && rng::random_range(0.0..=1.0).unwrap_or(1.0) <= target_specs.block;

    if is_blocked {
        target_state.just_blocked = true;
        events_queue.register_event(GameEvent::Block(damage_event.clone()));
        return;
    }

    if amount <= 0.0 {
        return;
    }

    if is_crit {
        target_state.just_hurt_crit = true;
        events_queue.register_event(GameEvent::CriticalStrike(damage_event.clone()));
    }

    target_state.health = (target_state.health - amount)
        .max(0.0)
        .min(target_specs.max_life);

    target_state.just_hurt = true;
    events_queue.register_event(GameEvent::Hit(damage_event));
}

pub fn heal_character(target: &mut Target, amount: f64) {
    let (_, (target_specs, target_state)) = target;

    if amount <= 0.0 {
        return;
    }

    if target_state.is_alive {
        target_state.health = (target_state.health + amount)
            .max(0.0)
            .min(target_specs.max_life);
    }
}

pub fn apply_status(target: &mut Target, status_type: StatusType, value: f64, duration: f64) {
    let (_, (target_specs, target_state)) = target;

    if duration <= 0.0 || !target_state.is_alive {
        return;
    }

    let mut value = value;

    match status_type {
        StatusType::Stunned => {}
        StatusType::DamageOverTime(damage_type) => {
            value = decrease_damage_from_armor(value, damage_type, target_specs);
        }
    }

    target_state
        .statuses
        .entry(status_type)
        .and_modify(|cur_status| {
            if value * duration > cur_status.value * cur_status.duration {
                cur_status.value = value;
                cur_status.duration = duration;
            }
        })
        .or_insert(StatusState { value, duration });
}

fn decrease_damage_from_armor(
    amount: f64,
    damage_type: DamageType,
    target_specs: &CharacterSpecs,
) -> f64 {
    amount
        * match damage_type {
            DamageType::Physical => {
                1.0 - increase_factors::diminishing(
                    target_specs.armor,
                    increase_factors::ARMOR_FACTOR,
                )
            }
            DamageType::Fire => {
                1.0 - increase_factors::diminishing(
                    target_specs.fire_armor,
                    increase_factors::ARMOR_FACTOR,
                )
            }
            DamageType::Poison => {
                1.0 - increase_factors::diminishing(
                    target_specs.poison_armor,
                    increase_factors::ARMOR_FACTOR,
                )
            }
        }
}
