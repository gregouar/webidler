use std::{collections::HashMap, time::Duration};

use shared::data::{
    character::{CharacterAttrs, CharacterId},
    character_status::{StatusEffectType, StatusMap, StatusSpecs, StatusState},
    player::CharacterState,
    skill::SkillType,
    stat_effect::StatEffect,
    values::NonNegative,
};

use crate::game::data::master_store::StatusesStore;

use super::characters_controller;

pub fn update_character_statuses(
    statuses_store: &StatusesStore,
    character_attrs: &CharacterAttrs,
    character_state: &mut CharacterState,
    elapsed_time: Duration,
) {
    let elapsed_time_f64 = elapsed_time.as_secs_f64().into();

    character_state.statuses.retain(|status_id, status_stacks| {
        let Some(status_specs) = statuses_store.get(status_id) else {
            return false;
        };

        status_stacks.retain_mut(|status_state| {
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
        !status_stacks.is_empty()
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
    if status_state.escalation.get() > 0.0 {
        // status_state.value = status_state.value
        //     * (1.0 + status_state.escalation * 0.01).powf(elapsed_time_f64.get());
        status_state.value = status_state.base_value
            * (1.0 + status_state.escalation.get() * 0.01 * status_state.elapsed_escalation.get());

        status_state.elapsed_escalation += elapsed_time_f64;

        if status_state.elapsed_escalation.get() > status_state.max_escalation.get() {
            status_state.elapsed_escalation = status_state.max_escalation;
        }
    }

    for status_effect in status_specs.effects.iter() {
        if let StatusEffectType::DamageOverTime { damage_type } = &status_effect.status_effect_type
        {
            characters_controller::damage_character(
                character_attrs,
                character_life,
                character_mana,
                &HashMap::from([(
                    *damage_type,
                    (status_effect.computed_value(status_state.value)
                        * elapsed_time_f64.get().min(status_state.duration.get())),
                )]),
                status_state.skill_type,
                false,
            );
        }
    }

    status_state.duration -= elapsed_time_f64;

    let remove_status = status_state.duration.get() <= 0.0;

    if remove_status
        && status_specs.effects.iter().any(|status_effect| {
            matches!(
                status_effect.status_effect_type,
                StatusEffectType::StatModifier { .. } | StatusEffectType::Trigger { .. }
            )
        })
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

pub fn generate_effects_from_statuses(
    statuses_store: &StatusesStore,
    statuses: &StatusMap,
) -> Vec<StatEffect> {
    statuses
        .iter()
        .filter_map(|(status_id, status_stacks)| {
            statuses_store
                .get(status_id)
                .map(|status_specs| (status_specs, status_stacks))
        })
        .flat_map(|(status_specs, status_stacks)| {
            status_specs.effects.iter().filter_map(|status_effect| {
                if let StatusEffectType::StatModifier {
                    stat,
                    modifier,
                    debuff,
                } = &status_effect.status_effect_type
                {
                    Some(StatEffect {
                        stat: stat.clone(),
                        modifier: *modifier,
                        value: status_stacks
                            .iter()
                            .map(|status_state| {
                                status_effect.computed_value(status_state.value).get()
                            })
                            .sum::<f64>()
                            * if *debuff { -1.0 } else { 1.0 },
                        bypass_ignore: false,
                    })
                } else {
                    None
                }
            })
        })
        .collect()

    // for (status_id, status_stacks) in statuses.iter() {
    //     let Some(status_specs) = statuses_store.get(status_id) else {
    //         continue;
    //     };

    //     for status_effect in status_specs.effects.iter() {
    //         if let StatusEffectType::StatModifier {
    //             stat,
    //             modifier,
    //             debuff,
    //         } = &status_effect.status_effect_type
    //         {
    //             *effects_map
    //                 .entry((stat.clone(), *modifier, false))
    //                 .or_default() += status_stacks
    //                 .iter()
    //                 .map(|status_state| status_effect.computed_value(status_state.value).get())
    //                 .sum::<f64>()
    //                 * if *debuff { -1.0 } else { 1.0 };
    //         }
    //     }
    // }
}

pub fn initialize_status_state(
    owner: CharacterId,
    skill_type: SkillType,
    value: NonNegative,
    duration: NonNegative,
    escalation: NonNegative,
) -> StatusState {
    StatusState {
        owner,
        value,
        duration,
        skill_type,
        base_value: value,
        elapsed_escalation: Default::default(),
        max_escalation: duration,
        escalation,
    }
}
