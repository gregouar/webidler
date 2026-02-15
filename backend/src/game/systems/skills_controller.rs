use std::collections::{HashMap, HashSet};

use rand::{self, seq::IteratorRandom};

use shared::{
    computations::skill_cost_increase,
    data::{
        character::{CharacterId, SkillSpecs, SkillState},
        item::{SkillRange, SkillShape},
        player::PlayerResources,
        skill::{
            SkillEffect, SkillEffectType, SkillRepeatTarget, SkillTargetsGroup, SkillType,
            TargetType,
        },
    },
};

use crate::game::{
    data::event::EventsQueue,
    systems::{skills_updater, stats_updater},
    utils::{
        rng::{self, flip_coin, RngSeed, Rollable},
        AnyAll,
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
) -> f64 {
    if !skill_state.is_ready || me.1 .1.mana < skill_specs.mana_cost.evaluate() {
        return me.1 .1.mana;
    }

    let mut applied = false;
    for targets_group in skill_specs.targets.iter() {
        applied |= apply_skill_on_targets(
            events_queue,
            skill_specs.base.skill_type,
            targets_group,
            me,
            friends,
            enemies,
        );
    }

    if applied {
        characters_controller::spend_mana(me.1 .0, me.1 .1, skill_specs.mana_cost.evaluate());
        skill_state.just_triggered = true;
        skill_state.is_ready = false;
        skill_state.elapsed_cooldown = 0.0;
    }

    characters_controller::mana_available(me.1 .0, me.1 .1)
}

fn apply_skill_on_targets<'a>(
    events_queue: &mut EventsQueue,
    skill_type: SkillType,
    targets_group: &SkillTargetsGroup,
    me: &mut Target<'a>,
    friends: &mut [Target<'a>],
    enemies: &mut [Target<'a>],
) -> bool {
    let mut already_hit = HashSet::new();

    for _ in 0..targets_group.repeat.value.roll() {
        match apply_repeated_skill_on_targets(
            events_queue,
            skill_type,
            targets_group,
            me,
            friends,
            enemies,
            &already_hit,
        ) {
            Some(id) => {
                already_hit.insert(id);
            }
            None => break,
        }
    }

    !already_hit.is_empty()
}

fn apply_repeated_skill_on_targets<'a>(
    events_queue: &mut EventsQueue,
    skill_type: SkillType,
    targets_group: &SkillTargetsGroup,
    me: &mut Target<'a>,
    friends: &mut [Target<'a>],
    enemies: &mut [Target<'a>],
    already_hit: &HashSet<CharacterId>,
) -> Option<CharacterId> {
    let attacker = me.0;

    let (main_target_id, mut targets) = {
        match targets_group.target_type {
            TargetType::Enemy => find_targets(
                targets_group,
                (me.1 .0.position_x, me.1 .0.position_y),
                enemies,
                already_hit,
            ),
            TargetType::Friend => find_targets(
                targets_group,
                (me.1 .0.position_x, me.1 .0.position_y),
                friends,
                already_hit,
            ),
            TargetType::Me => Some((me.0, vec![me])),
        }
    }?;

    let mut applied = false;
    for skill_effect in targets_group.effects.iter() {
        applied |= apply_skill_effect(
            events_queue,
            attacker,
            skill_type,
            targets_group.range,
            skill_effect,
            &mut targets,
            false,
        );
    }

    applied.then_some(main_target_id)
}

fn find_targets<'a, 'b>(
    targets_group: &SkillTargetsGroup,
    me_position: (u8, u8),
    pre_targets: &'b mut [Target<'a>],
    already_hit: &HashSet<CharacterId>,
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
    already_hit: &HashSet<CharacterId>,
) -> Option<(CharacterId, (u8, u8))> {
    // Filter by alive status & already hit targets depending on repeat type
    let target_specs = pre_targets
        .iter()
        .filter(|(_, (_, state))| {
            targets_group.target_dead != (state.is_alive & (state.life > 0.0))
        })
        .filter(|(id, _)| match targets_group.repeat.target {
            SkillRepeatTarget::Any => true,
            SkillRepeatTarget::Same => already_hit.is_empty() || already_hit.contains(id),
            SkillRepeatTarget::Different => !already_hit.contains(id),
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
            if flip_coin() {
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

// Return whether the skill could applied (like not enemy already cursed)
pub fn apply_skill_effect(
    events_queue: &mut EventsQueue,
    attacker: CharacterId,
    skill_type: SkillType,
    range: SkillRange,
    skill_effect: &SkillEffect,
    targets: &mut [&mut Target],
    is_triggered: bool,
) -> bool {
    let seed = rng::roll_seed();

    targets.iter_mut().any_all(|target| {
        apply_skill_effect_on_target(
            events_queue,
            attacker,
            skill_type,
            range,
            // TODO: Could branch to only clone when needed
            &apply_conditional_modifiers(target, skill_effect, skill_type),
            target,
            is_triggered,
            &mut seed.clone(),
        )
    })
}

fn apply_conditional_modifiers(
    target: &mut Target,
    skill_effect: &SkillEffect,
    skill_type: SkillType,
) -> SkillEffect {
    let mut new_skill_effect = skill_effect.clone();

    skills_updater::compute_skill_specs_effect(
        skill_type,
        &mut new_skill_effect,
        stats_updater::compute_conditional_modifiers(
            target.1 .0,
            target.1 .1,
            &skill_effect.conditional_modifiers,
        )
        .iter(),
    );

    new_skill_effect
}

#[allow(clippy::too_many_arguments)]
fn apply_skill_effect_on_target(
    events_queue: &mut EventsQueue,
    attacker: CharacterId,
    skill_type: SkillType,
    range: SkillRange,
    skill_effect: &SkillEffect,
    target: &mut Target,
    is_triggered: bool,
    seed: &mut RngSeed,
) -> bool {
    if !skill_effect.success_chance.roll_with_seed(seed) {
        return true;
    }

    match &skill_effect.effect_type {
        SkillEffectType::FlatDamage {
            damage,
            crit_chance,
            crit_damage,
            ignore_armor: _,
        } => {
            let is_crit = crit_chance.roll_with_seed(seed);

            let damage: HashMap<_, _> = damage
                .iter()
                .map(|(damage_type, value)| {
                    (
                        *damage_type,
                        value.roll_with_seed(seed)
                            * (if is_crit {
                                1.0 + crit_damage.evaluate() * 0.01
                            } else {
                                1.0
                            }),
                    )
                })
                .collect();

            characters_controller::attack_character(
                events_queue,
                target,
                attacker,
                damage.clone(),
                skill_type,
                range,
                is_crit,
                is_triggered,
            );

            true
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
                return false;
            }

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
                        is_triggered,
                    )
                });

            true
        }
        SkillEffectType::Restore {
            restore_type,
            value,
            modifier,
        } => characters_controller::restore_character(
            target,
            *restore_type,
            value.roll_with_seed(seed),
            *modifier,
        ),
        SkillEffectType::Resurrect => characters_controller::resuscitate_character(target),
    }
}

pub fn level_up_skill(skill_specs: &mut SkillSpecs, player_resources: &mut PlayerResources) {
    if player_resources.gold < skill_specs.next_upgrade_cost {
        return;
    }

    player_resources.gold -= skill_specs.next_upgrade_cost;

    skill_specs.upgrade_level += 1;
    skill_specs.next_upgrade_cost = skill_cost_increase(skill_specs);
}
