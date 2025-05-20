use rand::{self, seq::IteratorRandom};

use shared::data::{
    character::{CharacterSpecs, CharacterState, SkillSpecs, SkillState},
    effect::EffectTarget,
    item::{Range, Shape},
    item_affix::StatEffect,
    player::PlayerResources,
    skill::{SkillEffect, SkillEffectType, SkillType, TargetType},
};

use crate::game::utils::{increase_factors, rng};

use super::{characters_controller, stats_controller::ApplyStatModifier};

pub fn use_skill<'a>(
    skill_specs: &SkillSpecs,
    skill_state: &mut SkillState,
    me: (&'a CharacterSpecs, &'a mut CharacterState),
    mut friends: Vec<(&'a CharacterSpecs, &'a mut CharacterState)>,
    mut enemies: Vec<(&'a CharacterSpecs, &'a mut CharacterState)>,
) -> bool {
    let me_position = (me.0.position_x, me.0.position_y);
    let mut me = vec![me];
    skill_specs.effects.iter().any(|skill_effect| {
        apply_skill_effect(
            skill_effect,
            skill_state,
            me_position,
            &mut me,
            &mut friends,
            &mut enemies,
        )
    })
}

fn apply_skill_effect<'a>(
    skill_effect: &SkillEffect,
    skill_state: &mut SkillState,
    me_position: (u8, u8),
    me: &mut Vec<(&'a CharacterSpecs, &'a mut CharacterState)>,
    friends: &mut Vec<(&'a CharacterSpecs, &'a mut CharacterState)>,
    enemies: &mut Vec<(&'a CharacterSpecs, &'a mut CharacterState)>,
) -> bool {
    let pre_targets = match skill_effect.target_type {
        TargetType::Enemy => enemies,
        TargetType::Friend => friends,
        TargetType::Me => me,
    };

    let main_target_distance = match skill_effect.range {
        Range::Melee => pre_targets
            .iter()
            .map(|(specs, _)| specs.position_x.abs_diff(me_position.0))
            .min(),
        Range::Distance => pre_targets
            .iter()
            .map(|(specs, _)| specs.position_x.abs_diff(me_position.0))
            .max(),
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
        None => return false,
    };

    let dx = match skill_effect.range {
        Range::Melee => 1,
        Range::Distance => -1,
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

    let targets = pre_targets.iter_mut().filter(|(specs, _)| {
        let (x_size, y_size) = specs.size.get_xy_size();
        (0..x_size as i32)
            .flat_map(|x| (0..y_size as i32).map(move |y| (x, y)))
            .any(|(x, y)| {
                is_target_in_range((specs.position_x as i32 + x, specs.position_y as i32 + y))
            })
    });

    let mut found_target = false;
    for (target_specs, target_state) in targets {
        found_target = true;

        match skill_effect.effect_type {
            SkillEffectType::FlatDamage {
                min,
                max,
                damage_type,
                crit_chances,
                crit_damage,
            } => {
                let is_crit = rng::random_range(0.0..=1.0).unwrap_or(1.0) <= crit_chances;
                if let Some(damage) = rng::random_range(min..=max).map(|d| {
                    if is_crit {
                        d * (1.0 + crit_damage)
                    } else {
                        d
                    }
                }) {
                    characters_controller::damage_character(
                        damage,
                        damage_type,
                        target_state,
                        target_specs,
                        is_crit,
                    );
                }
            }
            SkillEffectType::Heal { min, max } => {
                if let Some(damage) = rng::random_range(min..=max) {
                    characters_controller::heal_character(damage, target_state, target_specs);
                }
            }
        }
    }

    if found_target {
        skill_state.just_triggered = true;
        skill_state.is_ready = false;
        skill_state.elapsed_cooldown = 0.0;
    }
    found_target
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
    skill_specs.next_upgrade_cost +=
        10.0 * increase_factors::exponential(skill_specs.upgrade_level as f64);

    increase_skill_effect(skill_specs);

    true
}

fn increase_skill_effect(skill_specs: &mut SkillSpecs) {
    for effect in skill_specs.effects.iter_mut() {
        effect.increase_effect(1.2);
    }
}

pub fn update_skill_specs(skill_specs: &mut SkillSpecs, effects: &[StatEffect]) {
    skill_specs.effects = skill_specs.base.effects.clone();
    skill_specs.cooldown = skill_specs.base.cooldown.clone();
    skill_specs.mana_cost = skill_specs.base.mana_cost.clone();

    for effect in effects.iter() {
        match effect.stat {
            EffectTarget::GlobalAttackSpeed
                if matches!(
                    skill_specs.base.skill_type,
                    SkillType::Attack | SkillType::Weapon(_)
                ) =>
            {
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
        increase_skill_effect(skill_specs)
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
                min,
                max,
                damage_type,
                crit_chances,
                crit_damage,
            } => match effect.stat {
                EffectTarget::GlobalSpellPower => {
                    min.apply_effect(effect);
                    max.apply_effect(effect);
                }
                EffectTarget::GlobalDamage(damage_type2) if damage_type2 == *damage_type => {
                    min.apply_effect(effect);
                    max.apply_effect(effect);
                }
                EffectTarget::GlobalAttackDamage
                    if matches!(skill_type, SkillType::Attack | SkillType::Weapon(_)) =>
                {
                    min.apply_effect(effect);
                    max.apply_effect(effect);
                }
                EffectTarget::GlobalSpellDamage if skill_type == SkillType::Spell => {
                    min.apply_effect(effect);
                    max.apply_effect(effect);
                }
                EffectTarget::GlobalCritChances => {
                    crit_chances.apply_effect(effect);
                }
                EffectTarget::GlobalCritDamage => {
                    crit_damage.apply_effect(effect);
                }
                _ => {}
            },
            SkillEffectType::Heal { min, max } => match effect.stat {
                EffectTarget::GlobalSpellPower => {
                    min.apply_effect(effect);
                    max.apply_effect(effect);
                }
                _ => {}
            },
        }
    }
}
