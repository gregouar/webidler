use rand::{self, seq::IteratorRandom};

use shared::data::{
    character::{CharacterSpecs, CharacterState, SkillSpecs, SkillState},
    effect::EffectTarget,
    item::{Range, Shape},
    item_affix::StatEffect,
    player::PlayerResources,
    skill::{SkillEffect, SkillEffectType, SkillType, TargetType},
    status::StatusType,
};

use crate::game::utils::{
    increase_factors,
    rng::{self, flip_coin},
};

use super::{characters_controller, stats_controller::ApplyStatModifier};

type Target<'a> = (&'a CharacterSpecs, &'a mut CharacterState);

pub fn use_skill<'a>(
    skill_specs: &SkillSpecs,
    skill_state: &mut SkillState,
    me: (&'a CharacterSpecs, &'a mut CharacterState),
    mut friends: Vec<(&'a CharacterSpecs, &'a mut CharacterState)>,
    mut enemies: Vec<(&'a CharacterSpecs, &'a mut CharacterState)>,
) -> bool {
    let me_position = (me.0.position_x, me.0.position_y);
    let mut me = vec![me];
    skill_specs
        .effects
        .iter()
        .fold(false, |applied, skill_effect| {
            applied
                | use_skill_effect(
                    skill_specs.base.skill_type,
                    skill_effect,
                    skill_state,
                    me_position,
                    &mut me,
                    &mut friends,
                    &mut enemies,
                )
        })
}

fn use_skill_effect<'a>(
    skill_type: SkillType,
    skill_effect: &SkillEffect,
    skill_state: &mut SkillState,
    me_position: (u8, u8),
    me: &mut Vec<Target<'a>>,
    friends: &mut Vec<Target<'a>>,
    enemies: &mut Vec<Target<'a>>,
) -> bool {
    let pre_targets = match skill_effect.target_type {
        TargetType::Enemy => enemies,
        TargetType::Friend => friends,
        TargetType::Me => me,
    };

    let mut targets = find_targets(skill_effect, me_position, pre_targets);

    if targets.is_empty() {
        return false;
    }

    skill_state.just_triggered = true;
    skill_state.is_ready = false;
    skill_state.elapsed_cooldown = 0.0;

    if rng::random_range(0.0..=1.0).unwrap_or(1.0) <= skill_effect.failure_chances {
        return true;
    }

    apply_skill_effect(skill_type, skill_effect, &mut targets);

    true
}

fn find_targets<'a, 'b>(
    skill_effect: &SkillEffect,
    me_position: (u8, u8),
    pre_targets: &'b mut [Target<'a>],
) -> Vec<&'b mut Target<'a>> {
    let available_positions = pre_targets
        .iter()
        .map(|(specs, _)| specs.position_x.abs_diff(me_position.0));

    let main_target_distance = match skill_effect.range {
        Range::Melee => available_positions.min(),
        Range::Distance => available_positions.max(),
        Range::Any => available_positions.choose(&mut rand::rng()),
    };

    let main_target_pos = main_target_distance.and_then(|distance| {
        pre_targets
            .iter()
            .filter(|(specs, _)| specs.position_x.abs_diff(me_position.0) == distance)
            .choose(&mut rand::rng())
            .map(|(specs, _)| (specs.position_x as i32, specs.position_y as i32))
    });

    let main_target_pos = match main_target_pos {
        Some(p) => p,
        None => return vec![],
    };

    let dx = match skill_effect.range {
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
        match skill_effect.shape {
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
        .filter(|(specs, _)| {
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

            for (damage_type, (min, max)) in damage {
                if let Some(amount) = rng::random_range(*min..=*max).map(|d| {
                    if is_crit {
                        d * (1.0 + crit_damage)
                    } else {
                        d
                    }
                }) {
                    for (target_specs, target_state) in targets.iter_mut() {
                        characters_controller::damage_character(
                            amount,
                            *damage_type,
                            skill_type,
                            target_state,
                            target_specs,
                            is_crit,
                        );
                    }
                }
            }
        }
        SkillEffectType::Heal { min, max } => {
            if let Some(amount) = rng::random_range(*min..=*max) {
                for (target_specs, target_state) in targets.iter_mut() {
                    characters_controller::heal_character(amount, target_state, target_specs);
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
                for (target_specs, target_state) in targets.iter_mut() {
                    characters_controller::apply_status(
                        *status_type,
                        value,
                        duration,
                        target_state,
                        target_specs,
                    )
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
    for effect in skill_specs.effects.iter_mut() {
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
    skill_specs.effects = skill_specs.base.effects.clone();
    skill_specs.cooldown = skill_specs.base.cooldown;
    skill_specs.mana_cost = skill_specs.base.mana_cost;

    for effect in effects.iter() {
        match effect.stat {
            EffectTarget::GlobalAttackSpeed if skill_specs.base.skill_type == SkillType::Attack => {
                skill_specs.cooldown.apply_inverse_effect(effect);
            }

            EffectTarget::GlobalSpellSpeed if skill_specs.base.skill_type == SkillType::Spell => {
                skill_specs.cooldown.apply_inverse_effect(effect);
            }

            EffectTarget::GlobalSpeed => {
                skill_specs.cooldown.apply_inverse_effect(effect);
            }

            _ => {}
        }
    }

    for skill_effect in skill_specs.effects.iter_mut() {
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
    // TODO: Different increase options?
    for effect in effects.iter() {
        match &mut skill_effect.effect_type {
            SkillEffectType::FlatDamage {
                damage,
                crit_chances,
                crit_damage,
            } => {
                for (damage_type, (min, max)) in damage {
                    match effect.stat {
                        EffectTarget::GlobalDamage(damage_type2)
                            if damage_type2 == *damage_type =>
                        {
                            min.apply_effect(effect);
                            max.apply_effect(effect);
                        }
                        EffectTarget::GlobalAttackDamage if skill_type == SkillType::Attack => {
                            min.apply_effect(effect);
                            max.apply_effect(effect);
                        }
                        EffectTarget::GlobalSpellDamage | EffectTarget::GlobalSpellPower
                            if skill_type == SkillType::Spell =>
                        {
                            min.apply_effect(effect);
                            max.apply_effect(effect);
                        }
                        _ => {}
                    }
                }

                match effect.stat {
                    EffectTarget::GlobalCritChances => {
                        crit_chances.apply_effect(effect);
                    }
                    EffectTarget::GlobalCritDamage => {
                        crit_damage.apply_effect(effect);
                    }
                    _ => {}
                }
            }
            SkillEffectType::Heal { min, max } => {
                if effect.stat == EffectTarget::GlobalSpellPower {
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
                    EffectTarget::GlobalSpellDamage | EffectTarget::GlobalSpellPower
                        if skill_type == SkillType::Spell =>
                    {
                        min_value.apply_effect(effect);
                        max_value.apply_effect(effect);
                    }
                    EffectTarget::GlobalAttackDamage if skill_type == SkillType::Attack => {
                        min_value.apply_effect(effect);
                        max_value.apply_effect(effect);
                    }
                    EffectTarget::GlobalDamage(damage_type2) if damage_type2 == *damage_type => {
                        min_value.apply_effect(effect);
                        max_value.apply_effect(effect);
                    }
                    _ => {}
                },
            },
        }
    }
}
