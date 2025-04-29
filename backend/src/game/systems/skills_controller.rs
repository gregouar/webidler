use rand::{self, seq::IteratorRandom};

use shared::data::{
    character::{CharacterSpecs, CharacterState, SkillSpecs, SkillState},
    item::{Range, Shape},
    skill::TargetType,
};

use crate::rng;

use super::characters_controller;

pub fn use_skill(
    skill_specs: &SkillSpecs,
    skill_state: &mut SkillState,
    me: (&CharacterSpecs, &mut CharacterState),
    friends: Vec<(&CharacterSpecs, &mut CharacterState)>,
    enemies: Vec<(&CharacterSpecs, &mut CharacterState)>,
) -> bool {
    let me_position = (me.0.position_x, me.0.position_y);

    let mut pre_targets = match skill_specs.target_type {
        TargetType::Enemy => enemies,
        TargetType::Friend => friends,
        TargetType::Me => vec![me],
    };

    let main_target_distance = match skill_specs.range {
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

    let dx = match skill_specs.range {
        Range::Melee => 1,
        Range::Distance => -1,
    };

    let is_target_in_range = |pos: (i32, i32)| -> bool {
        match skill_specs.shape {
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
        if let Some(damage) = rng::random_range(skill_specs.min_damages..=skill_specs.max_damages) {
            characters_controller::damage_character(damage, target_state, target_specs);
        }
    }

    if found_target {
        skill_state.just_triggered = true;
        skill_state.is_ready = false;
        skill_state.elapsed_cooldown = 0.0;
    }
    found_target
}

// pub fn pick_targets(character_specs: &CharacterSpecs,  monsters: &mut Vec<(&mut MonsterState, &MonsterSpecs)>, range: Range, target_type: TargetType, shape: Shape) ->
