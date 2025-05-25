use std::collections::HashMap;

use rand::{self, seq::IteratorRandom};

use shared::data::{
    character::{CharacterId, SkillSpecs, SkillState},
    character_status::StatusType,
    item::{Range, Shape},
    item_affix::StatEffect,
    player::PlayerResources,
    skill::{DamageType, SkillEffect, SkillEffectType, SkillTargetsGroup, SkillType, TargetType},
    stat_effect::{DamageMap, StatType},
};

use crate::game::{
    data::event::EventsQueue,
    utils::{
        increase_factors,
        rng::{self, flip_coin},
    },
};

use super::{
    characters_controller, characters_controller::Target, stats_controller::ApplyStatModifier,
};

pub fn use_skill<'a>(
    events_queue: &mut EventsQueue,
    skill_specs: &SkillSpecs,
    skill_state: &mut SkillState,
    me: &mut Target<'a>,
    mut friends: &mut [Target<'a>],
    mut enemies: &mut [Target<'a>],
) -> bool {
    let mut applied = false;

    for targets_group in skill_specs.targets.iter() {
        applied = applied
            | apply_skill_on_targets(
                events_queue,
                skill_specs.base.skill_type,
                targets_group,
                me,
                &mut friends,
                &mut enemies,
            );
    }

    if applied {
        skill_state.just_triggered = true;
        skill_state.is_ready = false;
        skill_state.elapsed_cooldown = 0.0;
    }

    applied
}

fn apply_skill_on_targets<'a>(
    events_queue: &mut EventsQueue,
    skill_type: SkillType,
    targets_group: &SkillTargetsGroup,
    me: &mut Target<'a>,
    friends: &mut [Target<'a>],
    enemies: &mut [Target<'a>],
) -> bool {
    let attacker = me.0;

    let mut targets = {
        match targets_group.target_type {
            TargetType::Enemy => find_targets(
                targets_group,
                (me.1 .0.position_x, me.1 .0.position_y),
                enemies,
            ),
            TargetType::Friend => find_targets(
                targets_group,
                (me.1 .0.position_x, me.1 .0.position_y),
                friends,
            ),
            TargetType::Me => vec![me],
        }
    };

    if targets.is_empty() {
        return false;
    }

    for skill_effect in targets_group.effects.iter() {
        if rng::random_range(0.0..=1.0).unwrap_or(1.0) >= skill_effect.failure_chances {
            apply_skill_effect(
                events_queue,
                attacker,
                skill_type,
                skill_effect,
                &mut targets,
            );
        }
    }

    true
}

fn find_targets<'a, 'b>(
    targets_group: &SkillTargetsGroup,
    me_position: (u8, u8),
    pre_targets: &'b mut [Target<'a>],
) -> Vec<&'b mut Target<'a>> {
    let target_specs = pre_targets.iter().map(|(_, (specs, _))| specs);

    let available_positions = target_specs
        .clone()
        .map(|specs| specs.position_x.abs_diff(me_position.0));

    let main_target_distance = match targets_group.range {
        Range::Melee => available_positions.min(),
        Range::Distance => available_positions.max(),
        Range::Any => available_positions.choose(&mut rand::rng()),
    };

    let main_target_pos = main_target_distance.and_then(|distance| {
        target_specs
            .clone()
            .filter(|specs| specs.position_x.abs_diff(me_position.0) == distance)
            .choose(&mut rand::rng())
            .map(|specs| (specs.position_x as i32, specs.position_y as i32))
    });

    let main_target_pos = match main_target_pos {
        Some(p) => p,
        None => return vec![],
    };

    let dx = match targets_group.range {
        Range::Melee => 1,
        Range::Distance => -1,
        Range::Any => {
            if flip_coin() {
                1
            } else {
                -1
            }
        }
    };

    let is_target_in_range = |pos: (i32, i32)| -> bool {
        match targets_group.shape {
            Shape::Single => pos == main_target_pos,
            Shape::Vertical2 => pos.0 == main_target_pos.0 && (pos.1 == 1 || pos.1 == 2),
            Shape::Horizontal2 => {
                (pos.0 == main_target_pos.0 || pos.0 == main_target_pos.0 + dx)
                    && pos.1 == main_target_pos.1
            }
            Shape::Horizontal3 => {
                (pos.0 == main_target_pos.0
                    || pos.0 == main_target_pos.0 + dx
                    || pos.0 == main_target_pos.0 + 2 * dx)
                    && pos.1 == main_target_pos.1
            }
            Shape::Square4 => {
                (pos.0 == main_target_pos.0 || pos.0 == main_target_pos.0 + dx)
                    && (pos.1 == 1 || pos.1 == 2)
            }
            Shape::All => true,
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

fn apply_skill_effect(
    events_queue: &mut EventsQueue,
    attacker: CharacterId,
    skill_type: SkillType,
    skill_effect: &SkillEffect,
    targets: &mut [&mut Target],
) {
    match &skill_effect.effect_type {
        SkillEffectType::FlatDamage {
            damage,
            crit_chances,
            crit_damage,
        } => {
            let is_crit = rng::random_range(0.0..=1.0).unwrap_or(1.0) <= *crit_chances;

            let damage: HashMap<_, _> = damage
                .iter()
                .map(|(damage_type, (min, max))| {
                    (
                        *damage_type,
                        rng::random_range(*min..=*max)
                            .map(|d| if is_crit { d * (1.0 + crit_damage) } else { d })
                            .unwrap_or(0.0),
                    )
                })
                .collect();

            for target in targets {
                characters_controller::attack_character(
                    events_queue,
                    target,
                    attacker,
                    damage.clone(),
                    skill_type,
                    is_crit,
                );
            }
        }
        SkillEffectType::Heal { min, max } => {
            if let Some(amount) = rng::random_range(*min..=*max) {
                for target in targets {
                    characters_controller::heal_character(target, amount);
                }
            }
        }
        SkillEffectType::ApplyStatus {
            status_type,
            min_value,
            max_value,
            min_duration,
            max_duration,
        } => {
            if let (Some(value), Some(duration)) = (
                rng::random_range(*min_value..=*max_value),
                rng::random_range(*min_duration..=*max_duration),
            ) {
                for target in targets {
                    characters_controller::apply_status(target, *status_type, value, duration)
                }
            }
        }
    }
}

pub fn level_up_skill(
    skill_specs: &mut SkillSpecs,
    player_resources: &mut PlayerResources,
) -> bool {
    if player_resources.gold < skill_specs.next_upgrade_cost {
        return false;
    }

    player_resources.gold -= skill_specs.next_upgrade_cost;

    skill_specs.upgrade_level += 1;
    skill_specs.next_upgrade_cost += 10.0
        * increase_factors::exponential(
            skill_specs.upgrade_level,
            increase_factors::MONSTER_INCREASE_FACTOR,
        );

    // TODO: recall update_skill_specs
    increase_skill_effects(skill_specs, 1.2);

    true
}

pub fn increase_skill_effects(skill_specs: &mut SkillSpecs, factor: f64) {
    for effect in skill_specs
        .targets
        .iter_mut()
        .flat_map(|t| t.effects.iter_mut())
    {
        increase_effect(effect, factor);
    }
}

// TODO: figure out linear increase for Heal
fn increase_effect(effect: &mut SkillEffect, factor: f64) {
    match &mut effect.effect_type {
        SkillEffectType::FlatDamage { damage, .. } => {
            for (min, max) in damage.values_mut() {
                *min *= factor;
                *max *= factor;
            }
        }
        SkillEffectType::Heal { min, max } => {
            *min *= factor;
            *max *= factor;
        }
        SkillEffectType::ApplyStatus {
            min_value,
            max_value,
            ..
        } => {
            *min_value *= factor;
            *max_value *= factor;
        }
    }
}

pub fn update_skill_specs(skill_specs: &mut SkillSpecs, effects: &[StatEffect]) {
    skill_specs.targets = skill_specs.base.targets.clone();
    skill_specs.cooldown = skill_specs.base.cooldown;
    skill_specs.mana_cost = skill_specs.base.mana_cost;

    for effect in effects.iter() {
        match effect.stat {
            StatType::Speed(skill_type)
                if skill_specs.base.skill_type
                    == skill_type.unwrap_or(skill_specs.base.skill_type) =>
            {
                skill_specs.cooldown.apply_inverse_effect(effect);
            }
            _ => {}
        }
    }

    for skill_effect in skill_specs
        .targets
        .iter_mut()
        .flat_map(|t| t.effects.iter_mut())
    {
        compute_skill_specs_effect(skill_specs.base.skill_type, skill_effect, effects)
    }

    for _ in 1..skill_specs.upgrade_level {
        increase_skill_effects(skill_specs, 1.2);
    }
}

pub fn compute_skill_specs_effect(
    skill_type: SkillType,
    skill_effect: &mut SkillEffect,
    effects: &[StatEffect],
) {
    for effect in effects.iter() {
        match &mut skill_effect.effect_type {
            SkillEffectType::FlatDamage {
                damage,
                crit_chances,
                crit_damage,
            } => {
                match effect.stat {
                    StatType::SpellPower if skill_type == SkillType::Spell => {
                        for (min, max) in damage.values_mut() {
                            min.apply_effect(effect);
                            max.apply_effect(effect);
                        }
                    }
                    StatType::Damage {
                        skill_type: skill_type2,
                        damage_type,
                    } if skill_type == skill_type2.unwrap_or(skill_type) => {
                        apply_effect_on_damage(damage, damage_type, Some(effect), Some(effect))
                    }
                    StatType::MinDamage {
                        skill_type: skill_type2,
                        damage_type,
                    } if skill_type == skill_type2.unwrap_or(skill_type) => {
                        apply_effect_on_damage(damage, damage_type, Some(effect), None)
                    }
                    StatType::MaxDamage {
                        skill_type: skill_type2,
                        damage_type,
                    } if skill_type == skill_type2.unwrap_or(skill_type) => {
                        apply_effect_on_damage(damage, damage_type, None, Some(effect))
                    }
                    _ => {}
                }
                match effect.stat {
                    StatType::CritChances(skill_type2)
                        if skill_type == skill_type2.unwrap_or(skill_type) =>
                    {
                        crit_chances.apply_effect(effect);
                    }
                    StatType::CritDamage(skill_type2)
                        if skill_type == skill_type2.unwrap_or(skill_type) =>
                    {
                        crit_damage.apply_effect(effect);
                    }
                    _ => {}
                }

                *crit_chances = crit_chances.clamp(0.0, 1.0);
                damage.retain(|_, (min, max)| {
                    *min = min.clamp(0.0, *max);
                    *max > 0.0
                });
            }
            SkillEffectType::Heal { min, max } => {
                if effect.stat == StatType::SpellPower {
                    min.apply_effect(effect);
                    max.apply_effect(effect);
                }
            }
            SkillEffectType::ApplyStatus {
                status_type,
                min_value,
                max_value,
                ..
            } => match status_type {
                StatusType::Stunned => {
                    // Something?
                }
                StatusType::DamageOverTime(damage_type) => match effect.stat {
                    StatType::SpellPower if skill_type == SkillType::Spell => {
                        min_value.apply_effect(effect);
                        max_value.apply_effect(effect);
                    }
                    StatType::Damage {
                        skill_type: skill_type2,
                        damage_type: damage_type2,
                    } if skill_type == skill_type2.unwrap_or(skill_type)
                        && *damage_type == damage_type2.unwrap_or(*damage_type) =>
                    {
                        min_value.apply_effect(effect);
                        max_value.apply_effect(effect);
                    }
                    _ => {}
                },
            },
        }
    }
}

fn apply_effect_on_damage(
    damage: &mut DamageMap,
    damage_type: Option<DamageType>,
    min_effect: Option<&StatEffect>,
    max_effect: Option<&StatEffect>,
) {
    match damage_type {
        Some(damage_type) => {
            let (min, max) = damage.entry(damage_type).or_insert((0.0, 0.0));
            if let Some(e) = min_effect {
                min.apply_effect(e);
            }
            if let Some(e) = max_effect {
                max.apply_effect(e);
            }
        }
        None => {
            for (min, max) in damage.values_mut() {
                if let Some(e) = min_effect {
                    min.apply_effect(e);
                }
                if let Some(e) = max_effect {
                    max.apply_effect(e);
                }
            }
        }
    }
}
