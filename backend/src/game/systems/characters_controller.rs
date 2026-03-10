use std::collections::HashMap;

use shared::{
    computations,
    constants::ARMOR_FACTOR,
    data::{
        character::{CharacterId, CharacterSpecs, CharacterState},
        character_status::{StatusId, StatusSpecs, StatusState},
        item::SkillRange,
        modifier::Modifier,
        skill::{DamageType, RestoreModifier, RestoreType, SkillType},
        stat_effect::{StatStatusType, StatType, compare_options},
        values::NonNegative,
    },
};

use crate::game::{
    data::event::{EventsQueue, GameEvent, HitEvent, StatusEvent},
    utils::rng::Rollable,
};

pub type Target<'a> = (CharacterId, (&'a CharacterSpecs, &'a mut CharacterState));

/// Return whether the target was hurt
#[allow(clippy::too_many_arguments)]
pub fn attack_character(
    events_queue: &mut EventsQueue,
    target: &mut Target,
    attacker: CharacterId,
    damage: HashMap<DamageType, NonNegative>,
    skill_type: SkillType,
    range: SkillRange,
    is_crit: bool,
    unblockable: bool,
    trigger_id: Option<&str>,
) -> bool {
    let (target_id, (target_specs, target_state)) = target;

    let is_blocked = if unblockable {
        false
    } else {
        target_specs
            .block
            .get(&skill_type)
            .map(|block| block.roll())
            .unwrap_or_default()
    };

    let is_hurt = damage_character(
        target_specs,
        &mut target_state.life,
        &mut target_state.mana,
        &damage,
        skill_type,
        is_blocked,
    ) > 0.0;

    if is_blocked {
        target_state.just_blocked = true;
    }

    if is_hurt {
        target_state.just_hurt = true;
        if is_crit {
            target_state.just_hurt_crit = true;
        }
    }

    // Couldn't find how to do this better
    let event_damage = if is_blocked {
        let block_factor = target_specs.block_damage.get() as f64 * 0.01;
        damage
            .into_iter()
            .map(|(damage_type, amount)| (damage_type, amount * block_factor))
            .collect()
    } else {
        damage
    };

    events_queue.register_event(GameEvent::Hit(HitEvent {
        source: attacker,
        target: *target_id,
        skill_type,
        range,
        damage: event_damage,
        is_crit,
        is_blocked,
        is_hurt,
        trigger_id: trigger_id.map(String::from),
    }));

    is_hurt
}

pub fn damage_character(
    character_specs: &CharacterSpecs,
    life: &mut NonNegative,
    mana: &mut NonNegative,
    damage: &HashMap<DamageType, NonNegative>,
    skill_type: SkillType,
    is_blocked: bool,
) -> f64 {
    let amount = damage
        .iter()
        .map(|(damage_type, amount)| {
            compute_damage(
                character_specs,
                *amount,
                *damage_type,
                skill_type,
                is_blocked,
            )
            .get()
        })
        .sum();

    if amount <= 0.0 {
        return 0.0;
    }

    let take_from_mana = mana
        .get()
        .min(amount * (character_specs.take_from_mana_before_life.get() as f64 * 0.01));
    let take_from_life: f64 = amount - take_from_mana;

    *mana -= take_from_mana.into();
    *life -= take_from_life.into();

    amount
}

fn compute_damage(
    character_specs: &CharacterSpecs,
    amount: NonNegative,
    damage_type: DamageType,
    skill_type: SkillType,
    is_blocked: bool,
) -> NonNegative {
    let resistance_factor = (1.0
        - *character_specs
            .damage_resistance
            .get(&(skill_type, damage_type))
            .cloned()
            .unwrap_or_default()
            * 0.01)
        .max(0.0);

    let armor_factor = (1.0
        - computations::diminishing(
            *character_specs
                .armor
                .get(&damage_type)
                .cloned()
                .unwrap_or_default(),
            ARMOR_FACTOR,
        ))
    .max(0.0);

    let block_factor = if is_blocked {
        character_specs.block_damage.get() as f64 * 0.01
    } else {
        1.0
    };

    (resistance_factor * armor_factor * block_factor * amount.get()).into()
}

pub fn restore_character(
    target: &mut Target,
    restore_type: RestoreType,
    amount: f64,
    modifier: RestoreModifier,
) -> bool {
    let (_, (target_specs, target_state)) = target;

    if !target_state.is_alive {
        return false;
    }

    let factor = match modifier {
        RestoreModifier::Percent => match restore_type {
            RestoreType::Life => target_specs.max_life.get() * 0.01,
            RestoreType::Mana => target_specs.max_mana.get() * 0.01,
        },
        RestoreModifier::Flat => 1.0,
    };

    match restore_type {
        RestoreType::Life => {
            if target_state.life.get() < target_specs.max_life.get() || amount < 0.0 {
                target_state.life += (amount * factor).into();
                true
            } else {
                false
            }
        }
        RestoreType::Mana => {
            if target_state.mana.get() < target_specs.max_mana.get() || amount < 0.0 {
                target_state.mana += (amount * factor).into();
                true
            } else {
                false
            }
        }
    }
}

pub fn resuscitate_character(target: &mut Target) -> bool {
    let (_, (target_specs, target_state)) = target;
    if target_state.is_alive {
        return false;
    }

    target_state.is_alive = true;
    target_state.life = target_specs.max_life.get().into();

    target_state
        .statuses
        .unique_statuses
        .retain(|_, (_, status_state)| status_state.duration.is_none());
    target_state
        .statuses
        .cumulative_statuses
        .retain(|(_, status_state)| status_state.duration.is_none());

    true
}

pub fn should_apply_status(
    target: &Target,
    status_specs: &StatusSpecs,
    skill_type: SkillType,
    value: NonNegative,
    duration: Option<NonNegative>,
    cumulate: bool,
    replace_on_value_only: bool,
) -> bool {
    let (_, (_, target_state)) = target;

    if duration.map(|d| d.get() <= 0.1).unwrap_or_default() || !target_state.is_alive {
        return false;
    }

    match status_specs {
        StatusSpecs::DamageOverTime { .. } | StatusSpecs::StatModifier { .. } => {
            if value.get() <= 0.0 {
                return false;
            }
        }
        StatusSpecs::Trigger(_) | StatusSpecs::Stun => {}
    }

    if cumulate {
        return true;
    }

    if let Some((_, cur_status_state)) = target_state
        .statuses
        .unique_statuses
        .get(&(status_specs.into(), skill_type))
    {
        return compute_effect_weight(value, duration, replace_on_value_only)
            > compute_effect_weight(
                cur_status_state.value,
                cur_status_state.duration,
                replace_on_value_only,
            );
    }

    true
}

#[allow(clippy::too_many_arguments)]
pub fn apply_status(
    events_queue: &mut EventsQueue,
    target: &mut Target,
    attacker: CharacterId,
    status_specs: &StatusSpecs,
    skill_type: SkillType,
    value: NonNegative,
    duration: Option<NonNegative>,
    cumulate: bool,
    unavoidable: bool,
    trigger_id: Option<&str>,
) -> bool {
    let (target_id, (target_specs, target_state)) = target;

    let status_resistance: f64 = target_specs
        .status_resistances
        .iter()
        .filter_map(|((res_skill_type, res_status_type), resistance)| {
            ((*res_skill_type == skill_type)
                && compare_options(res_status_type, &Some(status_specs.into())))
            .then_some(**resistance)
        })
        .sum();

    let (duration, value) = if status_resistance > 0.0 {
        let factor = (1.0 - status_resistance * 0.01).clamp(0.0, 1.0);
        if let Some(duration) = duration
            && duration.get() < 1e10
        {
            (Some(duration * factor), value)
        } else {
            (None, value * factor)
        }
    } else {
        (duration, value)
    };

    if duration.map(|d| d.get() <= 0.1).unwrap_or_default() || !target_state.is_alive {
        return false;
    }

    let is_evaded =
        if !unavoidable && let StatusSpecs::DamageOverTime { damage_type } = status_specs {
            target_specs
                .evade
                .get(damage_type)
                .map(|evade| evade.roll())
                .unwrap_or_default()
        } else {
            false
        };

    if is_evaded {
        target_state.just_evaded = true;
    }

    let evade_factor = if is_evaded {
        target_specs.evade_damage.get() as f64 * 0.01
    } else {
        1.0
    };

    let value = value * evade_factor;

    match status_specs {
        StatusSpecs::DamageOverTime { .. } | StatusSpecs::StatModifier { .. } => {
            if value.get() <= 0.0 {
                return false;
            }
        }
        StatusSpecs::Trigger(_) | StatusSpecs::Stun => {}
    }

    // Long duration are considered as forever
    let duration = match duration {
        Some(duration) if duration.get() > 9999.0f64 => None,
        _ => duration,
    };

    let mut new_status_specs = status_specs.clone();
    if let StatusSpecs::Trigger(ref mut trigger_specs) = new_status_specs {
        trigger_specs.triggered_effect.owner = Some(attacker);
    }

    let mut applied = true;
    if cumulate {
        target_state.statuses.cumulative_statuses.push((
            new_status_specs,
            StatusState {
                value,
                duration,
                cumulate,
                skill_type,
            },
        ));

        // TODO: Quickfix, have proper limit later
        if target_state.statuses.cumulative_statuses.len() > 100 {
            let status_id: StatusId = status_specs.into();

            if let Some(i) = target_state
                .statuses
                .cumulative_statuses
                .iter()
                .enumerate()
                .rev()
                .filter(|(_, (specs, _))| StatusId::from(specs) == status_id)
                .nth(100)
                .map(|(i, _)| i)
            {
                target_state.statuses.cumulative_statuses.remove(i);
            }
        }
    } else {
        target_state
            .statuses
            .unique_statuses
            .entry((status_specs.into(), skill_type))
            .and_modify(|(cur_status_specs, cur_status_state)| {
                if compute_effect_weight(value, duration, false)
                    > compute_effect_weight(
                        cur_status_state.value,
                        cur_status_state.duration,
                        false,
                    )
                {
                    cur_status_state.value = value;
                    cur_status_state.duration = duration;
                    *cur_status_specs = new_status_specs.clone();
                } else {
                    applied = false;
                }
            })
            .or_insert((
                new_status_specs,
                StatusState {
                    value,
                    duration,
                    cumulate,
                    skill_type,
                },
            ));
    }

    if !applied {
        return false;
    }

    events_queue.register_event(GameEvent::StatusApplied(StatusEvent {
        source: attacker,
        target: *target_id,
        skill_type,
        status_type: status_specs.into(),
        value,
        duration,
        trigger_id: trigger_id.map(String::from),
    }));

    if let StatusSpecs::StatModifier { .. } | StatusSpecs::Trigger { .. } = status_specs {
        target_state.dirty_specs = true;
    }

    let stun_lockout = *target_specs.stun_lockout;
    if let StatusSpecs::Stun = status_specs
        && stun_lockout.get() > 0.0
    {
        apply_status(
            events_queue,
            target,
            attacker,
            &StatusSpecs::StatModifier {
                stat: StatType::StatusResistance {
                    skill_type: None,
                    status_type: Some(StatStatusType::Stun),
                },
                modifier: Modifier::Flat,
                debuff: false,
            },
            SkillType::Other,
            100.0.into(),
            duration.map(|d| d + stun_lockout),
            false,
            true,
            Some("stun_lockout"),
        );
    }

    true
}

fn compute_effect_weight(
    value: NonNegative,
    duration: Option<NonNegative>,
    value_only: bool,
) -> f64 {
    let value = value.get() + 1.0;
    if value_only {
        if duration.map(|x| x.get()).unwrap_or(1.0) < 0.2 {
            value * 0.1
        } else {
            value
        }
    } else {
        value * duration.map(|x| x.get()).unwrap_or(10_000.0)
    }
}

pub fn mana_available(
    character_specs: &CharacterSpecs,
    character_state: &CharacterState,
) -> NonNegative {
    if character_specs.take_from_life_before_mana.get() > 0.0 {
        character_state.mana + (character_state.life.get() - 1.0).max(0.0).into()
    } else {
        character_state.mana
    }
}

pub fn spend_mana(
    character_specs: &CharacterSpecs,
    character_state: &mut CharacterState,
    amount: NonNegative,
) {
    let take_from_life = (character_state.life.get() - 1.0)
        .max(0.0)
        .min(amount.get() * (character_specs.take_from_life_before_mana.get() as f64 * 0.01));
    let take_from_mana = amount.get() - take_from_life;

    character_state.life -= take_from_life.into();
    character_state.mana -= take_from_mana.into();
}
