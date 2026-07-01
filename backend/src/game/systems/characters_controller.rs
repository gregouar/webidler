use std::collections::HashMap;

use shared::{
    computations,
    constants::ARMOR_FACTOR,
    data::{
        character::{CharacterAttrs, CharacterId, CharacterState},
        character_status::{StatusEffectType, StatusId},
        item::SkillRange,
        player::CharacterSpecs,
        skill::{DamageType, RestoreModifier, RestoreType, SkillType},
        stat_effect::{StatSkillFilter, compare_options},
        values::{Cooldown, NonNegative},
    },
};

use crate::game::{
    data::{
        event::{EventsQueue, GameEvent, HitEvent, StatusEvent},
        master_store::StatusesStore,
    },
    systems::statuses_controller,
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
    skill_id: &str,
    trigger_depth: u8,
) -> bool {
    let (target_id, (target_specs, target_state)) = target;

    let is_blocked = if unblockable {
        false
    } else {
        target_specs
            .character_attrs
            .block
            .get(&skill_type)
            .map(|block| block.roll())
            .unwrap_or_default()
    };

    let is_hurt = damage_character(
        &target_specs.character_attrs,
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
        let block_factor = target_specs.character_attrs.block_damage.get() as f64 * 0.01;
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
        skill_id: skill_id.into(),
        trigger_depth,
    }));

    !is_blocked
}

pub fn damage_character(
    character_attrs: &CharacterAttrs,
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
                character_attrs,
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
        .min(amount * (character_attrs.take_from_mana_before_life.get() as f64 * 0.01));
    let take_from_life: f64 = amount - take_from_mana;

    *mana -= take_from_mana.into();
    *life -= take_from_life.into();

    amount
}

fn compute_damage(
    character_attrs: &CharacterAttrs,
    amount: NonNegative,
    damage_type: DamageType,
    skill_type: SkillType,
    is_blocked: bool,
) -> NonNegative {
    let resistance_factor = (1.0
        - character_attrs
            .damage_resistance
            .get(&(skill_type, damage_type))
            .cloned()
            .unwrap_or_default()
            .get()
            * 0.01)
        .max(0.0);

    let armor_factor = (1.0
        - computations::diminishing(
            *character_attrs
                .armor
                .get(&damage_type)
                .cloned()
                .unwrap_or_default(),
            ARMOR_FACTOR,
        ))
    .max(0.0);

    let block_factor = if is_blocked {
        character_attrs.block_damage.get() as f64 * 0.01
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
            RestoreType::Life => target_specs.character_attrs.max_life.get() * 0.01,
            RestoreType::Mana => target_specs.character_attrs.max_mana.get() * 0.01,
        },
        RestoreModifier::Flat => 1.0,
    };

    match restore_type {
        RestoreType::Life => {
            if target_state.life.get() < target_specs.character_attrs.max_life.get() || amount < 0.0
            {
                target_state.life += (amount * factor).into();
                true
            } else {
                false
            }
        }
        RestoreType::Mana => {
            if target_state.mana.get() < target_specs.character_attrs.max_mana.get() || amount < 0.0
            {
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
    target_state.life = target_specs.character_attrs.max_life.get().into();
    target_state.resurrected = true;

    // TODO: Check if we needed that?
    // target_state
    //     .statuses
    //     .unique_statuses
    //     .retain(|_, (_, status_state)| status_state.duration.is_none());
    // target_state
    //     .statuses
    //     .cumulative_statuses
    //     .retain(|(_, status_state)| status_state.duration.is_none());
    target_state.statuses.clear();

    true
}

pub fn refresh_skills_cooldown(
    target: &mut Target,
    skill_filter: &StatSkillFilter,
    amount: f64,
    modifier: &RestoreModifier,
) -> bool {
    let mut refreshed = false;
    for (skill_specs, skill_state) in target
        .1
        .0
        .skills_specs
        .iter()
        .zip(target.1.1.skills_states.iter_mut())
    {
        if skill_filter.is_match_with_skill(skill_specs.skill_type, &skill_specs.skill_id) {
            match modifier {
                RestoreModifier::Flat => {
                    skill_state.elapsed_cooldown += Cooldown(amount / skill_specs.cooldown.get())
                }
                RestoreModifier::Percent => skill_state.elapsed_cooldown += Cooldown(amount * 0.01),
            }
            refreshed = true;
        }
    }

    refreshed
}

#[allow(clippy::too_many_arguments)]
pub fn should_apply_status(
    target: &Target,
    // status_specs: &StatusSpecs,
    status_id: &StatusId,
    // skill_type: SkillType,
    value: NonNegative,
    duration: NonNegative,
    escalation: NonNegative,
    max_stacks: u8,
    replace_on_value_only: bool,
) -> bool {
    let (_, (_, target_state)) = target;

    if duration.get() <= 0.1 || !target_state.is_alive {
        return false;
    }

    if value.get() <= 0.0 {
        return false;
    }

    // TODO: Smarter later?
    if max_stacks > 1 {
        return true;
    }

    target_state
        .statuses
        .get(status_id)
        .map(|cur_status_states| {
            cur_status_states.iter().any(|cur_status_state| {
                compute_effect_weight(value, duration, escalation, replace_on_value_only)
                    > compute_effect_weight(
                        cur_status_state.base_value,
                        cur_status_state.duration,
                        cur_status_state.escalation,
                        replace_on_value_only,
                    )
            })
        })
        .unwrap_or(true)
}

#[allow(clippy::too_many_arguments)]
pub fn apply_status(
    statuses_store: &StatusesStore,
    events_queue: &mut EventsQueue,
    target: &mut Target,
    attacker: CharacterId,
    status_id: &StatusId,
    skill_type: SkillType,
    value: NonNegative,
    duration: NonNegative,
    escalation: NonNegative,
    max_stacks: u8,
    avoidable: Option<DamageType>,
    skill_id: &str,
    trigger_depth: u8,
) -> bool {
    let (target_id, (target_specs, target_state)) = target;
    let status_id = statuses_store.id_with_key(status_id.clone());
    let Some(status_specs) = statuses_store.get(&status_id) else {
        return false;
    };

    let is_stun = status_id == "stun";

    let status_resistance: f64 = target_specs
        .character_attrs
        .status_resistances
        .iter()
        .filter_map(|((res_skill_type, res_status_id), resistance)| {
            ((*res_skill_type == skill_type)
                && compare_options(&res_status_id.as_ref(), &Some(&status_id)))
            .then_some(**resistance)
        })
        .sum();

    let factor = (1.0 - status_resistance * 0.01).clamp(0.0, 1.0);
    let duration = duration * factor;

    if duration.get() <= 0.1 || !target_state.is_alive {
        return false;
    }

    let is_evaded = if let Some(damage_type) = avoidable {
        target_specs
            .character_attrs
            .evade
            .get(&damage_type)
            .map(|evade| evade.roll())
            .unwrap_or_default()
    } else {
        false
    };

    if is_evaded {
        target_state.just_evaded = true;
    }

    let evade_factor = if is_evaded {
        target_specs.character_attrs.evade_damage.get() as f64 * 0.01
    } else {
        1.0
    };

    let value = value * evade_factor;

    let try_apply = value.get() > 0.0;
    let mut applied = try_apply;

    if try_apply {
        let status_stacks = target_state
            .statuses
            .entry(status_id.clone())
            .or_insert(Vec::with_capacity(max_stacks as usize));

        if status_stacks.len() < max_stacks as usize {
            status_stacks.push(statuses_controller::initialize_status_state(
                attacker, skill_type, value, duration, escalation,
            ));
        } else if let Some(cur_status_state) = status_stacks.iter_mut().find(|cur_status_state| {
            compute_effect_weight(value, duration, escalation, false)
                > compute_effect_weight(
                    cur_status_state.base_value,
                    cur_status_state.duration,
                    cur_status_state.escalation,
                    false,
                )
        }) {
            cur_status_state.owner = attacker;
            cur_status_state.base_value = value;
            cur_status_state.value = value;
            cur_status_state.duration = duration;
            cur_status_state.escalation = escalation;
            cur_status_state.max_escalation = duration;
            cur_status_state.skill_type = skill_type;
        } else {
            applied = false;
        }
    }

    if !applied && !is_evaded {
        return false;
    }

    if status_specs.effects.iter().any(|status_effect| {
        matches!(
            status_effect.status_effect_type,
            StatusEffectType::StatModifier { .. } | StatusEffectType::Trigger { .. }
        )
    }) {
        target_state.dirty_specs = true;
    }

    events_queue.register_event(GameEvent::StatusApplied(StatusEvent {
        source: attacker,
        target: *target_id,
        skill_type,
        status_id,
        damage_type: status_specs.damage_type,
        value,
        duration,
        is_evaded,
        skill_id: skill_id.into(),
        trigger_depth,
    }));

    let stun_lockout = *target_specs.character_attrs.stun_lockout;
    if is_stun && stun_lockout.get() > 0.0 {
        apply_status(
            statuses_store,
            events_queue,
            target,
            attacker,
            &"stun_lockout".into(), //This feels moronic
            SkillType::Other,
            100.0.into(),
            duration + stun_lockout,
            0.0.into(),
            1,
            None,
            "stun_lockout",
            0,
        );
    }

    applied
}

fn compute_effect_weight(
    value: NonNegative,
    duration: NonNegative,
    escalation: NonNegative,
    value_only: bool,
) -> f64 {
    let value = value.get() * (1.0 + escalation.get() * 0.01) + 1.0;
    if value_only {
        if duration.get() < 0.2 {
            value * 0.1
        } else {
            value
        }
    } else {
        value * duration.get().min(99999.0)
    }
}

pub fn mana_available(
    character_attrs: &CharacterAttrs,
    character_state: &CharacterState,
) -> NonNegative {
    if character_attrs.take_from_life_before_mana.get() > 0.0 {
        character_state.mana + (character_state.life.get() - 1.0).max(0.0).into()
    } else {
        character_state.mana
    }
}

pub fn spend_mana(
    character_attrs: &CharacterAttrs,
    character_state: &mut CharacterState,
    amount: NonNegative,
) {
    let take_from_life = (character_state.life.get() - 1.0)
        .max(0.0)
        .min(amount.get() * (character_attrs.take_from_life_before_mana.get() as f64 * 0.01));
    let take_from_mana = amount.get() - take_from_life;

    character_state.life -= take_from_life.into();
    character_state.mana -= take_from_mana.into();
}

pub fn reset_buff_statuses(statuses_store: &StatusesStore, character_state: &mut CharacterState) {
    character_state.statuses.retain(|status_id, _| {
        statuses_store
            .get(status_id)
            .map(|status_specs| status_specs.debuff)
            .unwrap_or_default()
    });
}
