use std::{collections::HashMap, time::Duration};

use shared::data::{
    character::CharacterAttrs,
    character_status::{StatusMap, StatusSpecs, StatusState},
    player::CharacterState,
    skill::SkillType,
    stat_effect::EffectsMap,
    values::NonNegative,
};

use super::characters_controller;

pub fn update_character_statuses(
    character_attrs: &CharacterAttrs,
    character_state: &mut CharacterState,
    elapsed_time: Duration,
) {
    let elapsed_time_f64 = elapsed_time.as_secs_f64().into();

    character_state
        .statuses
        .unique_statuses
        .retain(|_, (status_specs, status_state)| {
            update_status(
                character_attrs,
                &mut character_state.life,
                &mut character_state.mana,
                &mut character_state.dirty_specs,
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
                character_attrs,
                &mut character_state.life,
                &mut character_state.mana,
                &mut character_state.dirty_specs,
                status_specs,
                status_state,
                elapsed_time_f64,
            )
        });
}

fn update_status(
    character_attrs: &CharacterAttrs,
    character_life: &mut NonNegative,
    character_mana: &mut NonNegative,
    character_buff_status_change: &mut bool,
    status_specs: &StatusSpecs,
    status_state: &mut StatusState,
    elapsed_time_f64: NonNegative,
) -> bool {
    if status_state.escalation > 0.0 {
        // status_state.value = status_state.value
        //     * (1.0 + status_state.escalation * 0.01).powf(elapsed_time_f64.get());
        status_state.value = status_state.base_value
            * (1.0 + status_state.escalation * 0.01 * status_state.elapsed_escalation.get());

        status_state.elapsed_escalation += elapsed_time_f64;

        let max_escalation = status_state.max_escalation.unwrap_or(100.0.into());
        if status_state.elapsed_escalation.get() > max_escalation.get() {
            status_state.elapsed_escalation = max_escalation;
        }
    }

    if let StatusSpecs::DamageOverTime { damage_type } = status_specs {
        characters_controller::damage_character(
            character_attrs,
            character_life,
            character_mana,
            &HashMap::from([(
                *damage_type,
                (status_state.value.get()
                    * elapsed_time_f64
                        .get()
                        .min(status_state.duration.unwrap_or(elapsed_time_f64).get()))
                .into(),
            )]),
            status_state.skill_type,
            false,
        );
    }

    if let Some(duration) = status_state.duration.as_mut() {
        *duration -= elapsed_time_f64
    }

    let remove_status = status_state
        .duration
        .map(|d| d.get() <= 0.0)
        .unwrap_or_default();

    if let StatusSpecs::StatModifier { .. } | StatusSpecs::Trigger(_) = status_specs
        && remove_status
    {
        *character_buff_status_change = true;
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
                (stat.clone(), *modifier, false),
                if *debuff {
                    -status_state.value.get()
                } else {
                    status_state.value.get()
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

pub fn initialize_status_state(
    skill_type: SkillType,
    value: NonNegative,
    duration: Option<NonNegative>,
    escalation: f64,
    cumulate: bool,
) -> StatusState {
    StatusState {
        value,
        duration,
        cumulate,
        skill_type,
        base_value: value,
        elapsed_escalation: Default::default(),
        max_escalation: duration,
        escalation: escalation,
    }
}
