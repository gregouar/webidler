use rand::{self, seq::IteratorRandom};

use shared::data::{
    character::{CharacterSpecs, CharacterState, SkillSpecs, SkillState},
    item::{Range, Shape},
    player::PlayerResources,
    skill::{SkillEffect, SkillEffectType, TargetType},
};

use crate::rng;

use super::{characters_controller, increase_factors::exponential_factor};

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
            } => {
                if let Some(damage) = rng::random_range(min..=max) {
                    characters_controller::damage_character(
                        damage,
                        damage_type,
                        target_state,
                        target_specs,
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
    skill_specs.next_upgrade_cost += 10.0 * exponential_factor(skill_specs.upgrade_level as f64);

    for effect in skill_specs.effects.iter_mut() {
        effect.increase_effect(1.2);
    }

    true
}
