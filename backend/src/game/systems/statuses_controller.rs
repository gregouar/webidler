use std::{collections::HashMap, time::Duration};

use shared::data::{
    character_status::{StatusMap, StatusSpecs, StatusState},
    player::{CharacterSpecs, CharacterState},
    stat_effect::EffectsMap,
};

use super::characters_controller;

pub fn update_character_statuses(
    character_specs: &CharacterSpecs,
    character_state: &mut CharacterState,
    elapsed_time: Duration,
) {
    let elapsed_time_f64 = elapsed_time.as_secs_f64();

    character_state
        .statuses
        .unique_statuses
        .retain(|_, (status_specs, status_state)| {
            update_status(
                character_specs,
                &mut character_state.life,
                &mut character_state.mana,
                &mut character_state.buff_status_change,
                status_specs,
                status_state,
                elapsed_time_f64,
            )
        });

    character_state
        .statuses
        .cumulative_statuses
        .retain_mut(|(status_specs, status_state)| {
            update_status(
                character_specs,
                &mut character_state.life,
                &mut character_state.mana,
                &mut character_state.buff_status_change,
                status_specs,
                status_state,
                elapsed_time_f64,
            )
        });
}

fn update_status(
    character_specs: &CharacterSpecs,
    character_life: &mut f64,
    character_mana: &mut f64,
    character_buff_status_change: &mut bool,
    status_specs: &StatusSpecs,
    status_state: &mut StatusState,
    elapsed_time_f64: f64,
) -> bool {
    if let StatusSpecs::DamageOverTime { .. } = status_specs {
        characters_controller::damage_character(
            character_specs,
            character_life,
            character_mana,
            status_state.value
                * elapsed_time_f64.min(status_state.duration.unwrap_or(elapsed_time_f64)),
        );
    }

    if let Some(duration) = status_state.duration.as_mut() {
        *duration -= elapsed_time_f64
    }

    let remove_status = status_state.duration.unwrap_or(1.0) <= 0.0;

    if let StatusSpecs::StatModifier { .. } | StatusSpecs::Trigger(_) = status_specs {
        if remove_status {
            *character_buff_status_change = true;
        }
    }
    !remove_status
}

// pub fn generate_effects_map_from_statuses(statuses: &StatusMap) -> EffectsMap {
//     EffectsMap(
//         statuses
//             .iter()
//             .filter_map(|(status_specs, status_state)| match status_specs {
//                 StatusSpecs::StatModifier {
//                     stat,
//                     modifier,
//                     debuff,
//                 } => Some((
//                     (*stat, *modifier),
//                     if *debuff {
//                         -status_state.value
//                     } else {
//                         status_state.value
//                     },
//                 )),
//                 _ => None,
//             })
//             .collect(),
//     )
// }

pub fn generate_effects_map_from_statuses(statuses: &StatusMap) -> EffectsMap {
    statuses
        .iter()
        .filter_map(|(status_specs, status_state)| match status_specs {
            StatusSpecs::StatModifier {
                stat,
                modifier,
                debuff,
            } => Some((
                (stat.clone(), *modifier),
                if *debuff {
                    -status_state.value
                } else {
                    status_state.value
                },
            )),
            _ => None,
        })
        .fold(
            EffectsMap(HashMap::new()),
            |mut effects_map, (key, value)| {
                *effects_map.0.entry(key).or_default() += value;
                effects_map
            },
        )
}
