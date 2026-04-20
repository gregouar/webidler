use std::collections::{HashMap, HashSet};

use rand::{self, seq::IteratorRandom};

use shared::{
    computations::skill_cost_increase,
    data::{
        character::{CharacterId, SkillSpecs, SkillState},
        item::{SkillRange, SkillShape},
        player::PlayerResources,
        skill::{
            RepeatedSkillEffect, SkillEffect, SkillEffectType, SkillRepeatTarget,
            SkillTargetsGroup, SkillType, TargetType,
        },
        values::NonNegative,
    },
};

use crate::game::{
    data::event::EventsQueue,
    systems::{skills_updater, stats_updater},
    utils::{
        AnyAll,
        rng::{self, RngSeed, Rollable, flip_coin},
    },
};

use super::{characters_controller, characters_controller::Target};

/// Return remaining mana available
pub fn use_skill<'a>(
    events_queue: &mut EventsQueue,
    skill_specs: &SkillSpecs,
    skill_state: &mut SkillState,
    me: &mut Target<'a>,
    friends: &mut [Target<'a>],
    enemies: &mut [Target<'a>],
) -> NonNegative {
    if !skill_state.is_ready || me.1.1.mana.get() < skill_specs.mana_cost.get() {
        return me.1.1.mana;
    }

    let mut applied = false;
    for targets_group in skill_specs.targets.iter() {
        applied |= apply_skill_on_targets(
            events_queue,
            &skill_specs.base.skill_id,
            skill_specs.base.skill_type,
            targets_group,
            me,
            friends,
            enemies,
        );
    }

    if applied {
        characters_controller::spend_mana(me.1.0, me.1.1, *skill_specs.mana_cost);
        skill_state.just_triggered = true;
        skill_state.is_ready = false;
        skill_state.elapsed_cooldown = Default::default();
    }

    characters_controller::mana_available(me.1.0, me.1.1)
}

fn apply_skill_on_targets<'a>(
    events_queue: &mut EventsQueue,
    skill_id: &String,
    skill_type: SkillType,
    targets_group: &SkillTargetsGroup,
    me: &mut Target<'a>,
    friends: &mut [Target<'a>],
    enemies: &mut [Target<'a>],
) -> bool {
    let max_repeat = targets_group.repeat.value.roll();

    if max_repeat == 0 {
        return false;
    }

    let character_hit = apply_repeated_skill_on_targets(
        events_queue,
        skill_id,
        skill_type,
        targets_group,
        me,
        friends,
        enemies,
        None,
    );

    match character_hit {
        Some(character_hit) => {
            if max_repeat > 1 {
                me.1.1.repeated_skills.push(RepeatedSkillEffect {
                    skill_id: skill_id.clone(),
                    skill_type,
                    targets_group: targets_group.clone(),
                    max_repeat,
                    amount_repeat: 1,
                    elapsed_cooldown: Default::default(),
                    already_hit: HashSet::from([character_hit]),
                });
            }
            true
        }
        None => false,
    }
}

#[allow(clippy::too_many_arguments)]
fn apply_repeated_skill_on_targets<'a>(
    events_queue: &mut EventsQueue,
    skill_id: &String,
    skill_type: SkillType,
    targets_group: &SkillTargetsGroup,
    me: &mut Target<'a>,
    friends: &mut [Target<'a>],
    enemies: &mut [Target<'a>],
    already_hit: Option<&HashSet<CharacterId>>,
) -> Option<CharacterId> {
    let attacker = me.0;

    let (main_target_id, mut targets) = {
        match targets_group.target_type {
            TargetType::Enemy => find_targets(
                targets_group,
                (me.1.0.position_x, me.1.0.position_y),
                enemies,
                already_hit,
            ),
            TargetType::Friend => find_targets(
                targets_group,
                (me.1.0.position_x, me.1.0.position_y),
                friends,
                already_hit,
            ),
            TargetType::Me => Some((me.0, vec![me])),
        }
    }?;

    let applied = apply_skill_effects(
        events_queue,
        attacker,
        skill_id,
        skill_type,
        targets_group.range,
        &targets_group.effects,
        &mut targets,
        None,
    );

    applied.then_some(main_target_id)
}

fn find_targets<'a, 'b>(
    targets_group: &SkillTargetsGroup,
    me_position: (u8, u8),
    pre_targets: &'b mut [Target<'a>],
    already_hit: Option<&HashSet<CharacterId>>,
) -> Option<(CharacterId, Vec<&'b mut Target<'a>>)> {
    let (main_target_id, main_target_pos) =
        find_main_target(targets_group, me_position, pre_targets, already_hit)?;

    Some((
        main_target_id,
        find_sub_targets(
            targets_group.range,
            targets_group.shape,
            main_target_pos,
            (1, 1),
            pre_targets,
        ),
    ))
}

fn find_main_target<'a, 'b>(
    targets_group: &SkillTargetsGroup,
    me_position: (u8, u8),
    pre_targets: &'b mut [Target<'a>],
    already_hit: Option<&HashSet<CharacterId>>,
) -> Option<(CharacterId, (u8, u8))> {
    // Filter by alive status & already hit targets depending on repeat type
    let target_specs = pre_targets
        .iter()
        .filter(|(_, (_, state))| {
            targets_group.target_dead != (state.is_alive & (state.life.get() > 0.0))
        })
        .filter(|(id, _)| {
            already_hit
                .map(|already_hit| match targets_group.repeat.target {
                    SkillRepeatTarget::Any => true,
                    SkillRepeatTarget::Same => already_hit.is_empty() || already_hit.contains(id),
                    SkillRepeatTarget::Different => !already_hit.contains(id),
                })
                .unwrap_or(true)
        })
        .map(|(id, (specs, _))| (id, specs));

    // Pick closest/furthest target
    let available_positions = target_specs
        .clone()
        .map(|(_, specs)| specs.position_x.abs_diff(me_position.0));

    let main_target_distance = match targets_group.range {
        SkillRange::Melee => available_positions.min(),
        SkillRange::Distance => available_positions.max(),
        SkillRange::Any => available_positions.choose(&mut rand::rng()),
    };

    main_target_distance.and_then(|distance| {
        target_specs
            .clone()
            .filter(|(_, specs)| specs.position_x.abs_diff(me_position.0) == distance)
            .choose(&mut rand::rng())
            .map(|(id, specs)| {
                let (x_size, y_size) = specs.size.get_xy_size();
                let dx = rng::random_range(1..=x_size)
                    .and_then(|v| v.checked_sub(1))
                    .unwrap_or(0) as u8;
                let dy = rng::random_range(1..=y_size)
                    .and_then(|v| v.checked_sub(1))
                    .unwrap_or(0) as u8;
                (*id, (specs.position_x + dx, specs.position_y + dy))
            })
    })
}

pub fn find_sub_targets<'a, 'b>(
    skill_range: SkillRange,
    skill_shape: SkillShape,
    skill_position: (u8, u8),
    skill_size: (usize, usize),
    pre_targets: &'b mut [Target<'a>],
) -> Vec<&'b mut Target<'a>> {
    let skill_position = (skill_position.0 as i32, skill_position.1 as i32);
    let dx = match skill_range {
        SkillRange::Melee => 1,
        SkillRange::Distance => -1,
        SkillRange::Any => {
            if skill_position.0 <= 1 || (skill_position.0 == 2 && flip_coin()) {
                1
            } else {
                -1
            }
        }
    };

    // Check if the position is in AoE of skill
    let is_target_in_range = |pos: (i32, i32)| -> bool {
        match skill_shape {
            SkillShape::Single => pos == skill_position,
            SkillShape::Vertical2 => pos.0 == skill_position.0 && (pos.1 == 1 || pos.1 == 2),
            SkillShape::Horizontal2 => {
                (pos.0 == skill_position.0 || pos.0 == skill_position.0 + dx)
                    && pos.1 == skill_position.1
            }
            SkillShape::Horizontal3 => {
                (pos.0 == skill_position.0
                    || pos.0 == skill_position.0 + dx
                    || pos.0 == skill_position.0 + 2 * dx)
                    && pos.1 == skill_position.1
            }
            SkillShape::Square4 => {
                (pos.0 == skill_position.0 || pos.0 == skill_position.0 + dx)
                    && (pos.1 == 1 || pos.1 == 2)
            }
            SkillShape::All => true,
            SkillShape::Contact => {
                let x_dis = (pos.0 - skill_position.0)
                    .abs()
                    .min((pos.0 - skill_position.0 - skill_size.0 as i32 + 1).abs());
                let y_dis = (pos.1 - skill_position.1)
                    .abs()
                    .min((pos.1 - skill_position.1 - skill_size.1 as i32 + 1).abs());

                x_dis + y_dis == 1

                // ((pos.0 - skill_position.0).abs() <= 1
                //     || (pos.0 - skill_position.0 - skill_size.0 as i32 + 1).abs() <= 1)
                //     && ((pos.1 - skill_position.1).abs() <= 1
                //         || (pos.1 - skill_position.1 - skill_size.1 as i32 + 1).abs() <= 1)
            }
        }
    };

    pre_targets
        .iter_mut()
        .filter(|(_, (specs, _))| {
            let (x_size, y_size) = specs.size.get_xy_size();
            (0..x_size as i32)
                .flat_map(|x| (0..y_size as i32).map(move |y| (x, y)))
                .any(|(x, y)| {
                    is_target_in_range((specs.position_x as i32 + x, specs.position_y as i32 + y))
                })
        })
        .collect()
}

#[allow(clippy::too_many_arguments)]
pub fn apply_skill_effects(
    events_queue: &mut EventsQueue,
    attacker: CharacterId,
    skill_id: &String,
    skill_type: SkillType,
    range: SkillRange,
    skill_effects: &[SkillEffect],
    targets: &mut [&mut Target],
    trigger_id: Option<&str>,
) -> bool {
    let seed = rng::roll_seed();

    targets.iter_mut().any_all(|target| {
        let mut target_applicable = false;
        for skill_effect in skill_effects.iter() {
            let skill_effect = if skill_effect.conditional_modifiers.is_empty() {
                skill_effect
            } else {
                &apply_conditional_modifiers(target, skill_effect, skill_id, skill_type)
            };

            let (applicable, effective) = apply_skill_effect_on_target(
                events_queue,
                attacker,
                skill_type,
                range,
                skill_effect,
                target,
                trigger_id,
                &mut seed.clone(),
            );

            target_applicable |= applicable;
            if !effective {
                // In case of like hit blocked
                break;
            }
        }
        target_applicable
    })
}

fn apply_conditional_modifiers(
    target: &mut Target,
    skill_effect: &SkillEffect,
    skill_id: &String,
    skill_type: SkillType,
) -> SkillEffect {
    let mut new_skill_effect = skill_effect.clone();

    skills_updater::compute_skill_specs_effect(
        skill_id,
        skill_type,
        &mut new_skill_effect,
        stats_updater::compute_conditional_modifiers(
            &Default::default(),
            target.1.0,
            target.1.1,
            &skill_effect.conditional_modifiers,
        )
        .iter(),
    );

    new_skill_effect
}

/// Return first whether the skill effect was applicable
/// and secondly if it did applied (like not blocked)
#[allow(clippy::too_many_arguments)]
fn apply_skill_effect_on_target(
    events_queue: &mut EventsQueue,
    attacker: CharacterId,
    skill_type: SkillType,
    range: SkillRange,
    skill_effect: &SkillEffect,
    target: &mut Target,
    trigger_id: Option<&str>,
    seed: &mut RngSeed,
) -> (bool, bool) {
    let is_successful = skill_effect.success_chance.roll_with_seed(seed);

    match &skill_effect.effect_type {
        SkillEffectType::FlatDamage {
            damage,
            crit_chance,
            crit_damage,
            unblockable,
        } => {
            if !is_successful {
                return (true, false);
            }

            let is_crit = crit_chance.roll_with_seed(seed);

            let damage: HashMap<_, _> = damage
                .iter()
                .map(|(damage_type, value)| {
                    (
                        *damage_type,
                        (*value).roll_with_seed(seed)
                            * (if is_crit {
                                1.0 + **crit_damage * 0.01
                            } else {
                                1.0
                            }),
                    )
                })
                .collect();

            (
                true,
                characters_controller::attack_character(
                    events_queue,
                    target,
                    attacker,
                    damage.clone(),
                    skill_type,
                    range,
                    is_crit,
                    *unblockable,
                    trigger_id,
                ),
            )
        }
        SkillEffectType::ApplyStatus { duration, statuses } => {
            let values: Vec<_> = statuses
                .iter()
                .map(|status_effect| status_effect.value.roll_with_seed(seed))
                .collect();

            let duration = Some(duration.roll_with_seed(seed));

            if !statuses
                .iter()
                .zip(values.iter())
                .any(|(status_effect, value)| {
                    characters_controller::should_apply_status(
                        target,
                        &status_effect.status_type,
                        skill_type,
                        *value,
                        duration,
                        status_effect.cumulate,
                        status_effect.replace_on_value_only,
                    )
                })
            {
                return (false, false);
            }

            if !is_successful {
                return (true, false);
            }

            (
                true,
                statuses
                    .iter()
                    .zip(values.iter())
                    .any_all(|(status_effect, value)| {
                        characters_controller::apply_status(
                            events_queue,
                            target,
                            attacker,
                            &status_effect.status_type,
                            skill_type,
                            *value,
                            duration,
                            status_effect.cumulate,
                            status_effect.unavoidable,
                            trigger_id,
                        )
                    }),
            )
        }
        SkillEffectType::Restore {
            restore_type,
            value,
            modifier,
        } => {
            if !is_successful {
                return (true, false);
            }

            let restored = characters_controller::restore_character(
                target,
                *restore_type,
                value.roll_with_seed(seed),
                *modifier,
            );
            (restored, restored)
        }
        SkillEffectType::Resurrect => {
            if !is_successful {
                return (true, false);
            }

            let resurrected = characters_controller::resuscitate_character(target);
            (resurrected, resurrected)
        }
    }
}

pub fn repeat_skills<'a>(
    events_queue: &mut EventsQueue,
    me: &mut Target<'a>,
    friends: &mut [Target<'a>],
    enemies: &mut [Target<'a>],
) {
    let mut repeated_skills = std::mem::take(&mut me.1.1.repeated_skills);
    repeated_skills.retain_mut(|repeated_skill| {
        repeat_skill(events_queue, repeated_skill, me, friends, enemies)
    });
    me.1.1.repeated_skills = repeated_skills;
}

// Return whether the repeat effect is ended
fn repeat_skill<'a>(
    events_queue: &mut EventsQueue,
    repeated_skill_effect: &mut RepeatedSkillEffect,
    me: &mut Target<'a>,
    friends: &mut [Target<'a>],
    enemies: &mut [Target<'a>],
) -> bool {
    // TODO: Check cooldown
    if repeated_skill_effect
        .targets_group
        .repeat
        .repeat_cooldown
        .get()
        > 0.0
        && repeated_skill_effect.elapsed_cooldown.get() < 1.0
    {
        return true;
    }

    let character_hit = apply_repeated_skill_on_targets(
        events_queue,
        &repeated_skill_effect.skill_id,
        repeated_skill_effect.skill_type,
        &repeated_skill_effect.targets_group,
        me,
        friends,
        enemies,
        Some(&repeated_skill_effect.already_hit),
    );

    if let Some(charater_hit) = character_hit {
        repeated_skill_effect.already_hit.insert(charater_hit);
    }

    repeated_skill_effect.elapsed_cooldown =
        (repeated_skill_effect.elapsed_cooldown.get() - 1.0).into();
    repeated_skill_effect.amount_repeat += 1;

    repeated_skill_effect.amount_repeat < repeated_skill_effect.max_repeat
}

pub fn level_up_skill(skill_specs: &mut SkillSpecs, player_resources: &mut PlayerResources) {
    if player_resources.gold < skill_specs.next_upgrade_cost {
        return;
    }

    player_resources.gold -= skill_specs.next_upgrade_cost;

    skill_specs.upgrade_level += 1;
    skill_specs.next_upgrade_cost = skill_cost_increase(skill_specs);
}
