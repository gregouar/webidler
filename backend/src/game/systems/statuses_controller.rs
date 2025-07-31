use std::time::Duration;

use shared::data::{
    character_status::{StatusMap, StatusType},
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
        .retain(|status_type, status_states| {
            if let StatusType::DamageOverTime { .. } = status_type {
                for status in status_states.iter() {
                    characters_controller::damage_character(
                        character_specs,
                        &mut character_state.life,
                        &mut character_state.mana,
                        status.value * elapsed_time_f64.min(status.duration),
                    );
                }
            }

            let old_len = status_states.len();
            status_states.retain_mut(|status| {
                status.duration -= elapsed_time_f64;
                status.duration > 0.0
            });

            if let StatusType::StatModifier { .. } = status_type {
                if old_len != status_states.len() {
                    character_state.buff_status_change = true;
                }
            }

            !status_states.is_empty()
        });
}

pub fn generate_effects_map_from_statuses(statuses: &StatusMap) -> EffectsMap {
    EffectsMap(
        statuses
            .iter()
            .filter_map(|(s, v)| match s {
                StatusType::StatModifier {
                    stat,
                    modifier,
                    debuff,
                } => Some((
                    (*stat, *modifier),
                    v.iter()
                        .map(|s| if *debuff { -s.value } else { s.value })
                        .sum(),
                )),
                _ => None,
            })
            .collect(),
    )
}
