use std::time::Duration;

use shared::{
    constants::THREAT_EFFECT,
    data::{
        area::AreaThreat,
        character::CharacterId,
        character_status::StatusSpecs,
        modifier::Modifier,
        monster::{MonsterSpecs, MonsterState},
        stat_effect::{StatEffect, StatType},
    },
};

use crate::game::{
    data::event::EventsQueue,
    systems::{stats_updater, statuses_controller},
};

use super::{characters_updater, skills_updater};

pub fn update_monster_states(
    events_queue: &mut EventsQueue,
    elapsed_time: Duration,
    monster_specs: &[MonsterSpecs],
    monster_states: &mut [MonsterState],
    area_threat: &AreaThreat,
) {
    for (monster_id, (monster_state, monster_specs)) in monster_states
        .iter_mut()
        .zip(monster_specs.iter())
        .enumerate()
        .filter(|(_, (s, _))| s.character_state.is_alive)
    {
        characters_updater::update_character_state(
            events_queue,
            elapsed_time,
            CharacterId::Monster(monster_id),
            &monster_specs.character_specs,
            &mut monster_state.character_state,
            area_threat,
        );
    }
}

pub fn reset_monsters(monster_states: &mut [MonsterState]) {
    for monster_state in monster_states.iter_mut() {
        characters_updater::reset_character(&mut monster_state.character_state);
    }
}

pub fn update_monster_specs(
    base_specs: &MonsterSpecs,
    monster_specs: &mut MonsterSpecs,
    monster_state: &MonsterState,
    area_threat: &AreaThreat,
) {
    // let effects_map = EffectsMap::combine_all(
    //     std::iter::once(statuses_controller::generate_effects_map_from_statuses(
    //         &monster_state.character_state.statuses,
    //     ))
    //     .chain(std::iter::once(EffectsMap(HashMap::from([(
    //         (
    //             StatType::Damage {
    //                 skill_type: None,
    //                 damage_type: None,
    //                 min_max: None,
    //             },
    //             Modifier::Multiplier,
    //         ),
    //         ((1.0 + THREAT_EFFECT).powf(area_threat.threat_level as f64) - 1.0) * 100.0,
    //     )])))) // .chain(std::iter::once(base_specs.character_specs.effects.clone())),
    //     .chain(iter::once(
    //         stats_updater::compute_conditional_modifiers(
    //             area_threat,
    //             &monster_specs.character_specs,
    //             &monster_state.character_state,
    //             &monster_specs.character_specs.conditional_modifiers,
    //         )
    //         .into(),
    //     )),
    // );
    // let mut effects = (&effects_map).into();

    let mut effects: Vec<_> = (&statuses_controller::generate_effects_map_from_statuses(
        &monster_state.character_state.statuses,
    ))
        .into();
    effects.push(StatEffect {
        stat: StatType::Damage {
            skill_filter: Default::default(),
            damage_type: None,
            min_max: None,
            is_hit: None,
        },
        modifier: Modifier::Increased,
        value: ((1.0 + THREAT_EFFECT).powf(area_threat.threat_level as f64) - 1.0) * 100.0,
        bypass_ignore: false,
    });
    effects.extend(stats_updater::compute_conditional_modifiers(
        area_threat,
        &monster_specs.character_specs.character_attrs,
        &monster_state.character_state,
        &monster_specs.character_specs.conditional_modifiers,
    ));

    // Compute character specs & get converted stats resulting
    let (character_specs, converted_effects) =
        characters_updater::update_character_specs(&base_specs.character_specs, &effects);
    monster_specs.character_specs = character_specs;
    effects.extend(converted_effects);

    for skill_specs in monster_specs.character_specs.skills_specs.iter_mut() {
        skills_updater::apply_effects_to_skill_specs(skill_specs, effects.iter());
    }

    monster_specs.character_specs.triggers.extend(
        monster_state
            .character_state
            .statuses
            .iter()
            .filter_map(|(status_specs, _)| match status_specs {
                StatusSpecs::Trigger(trigger_specs) => Some(trigger_specs.as_ref()),
                _ => None,
            })
            .chain(
                monster_specs
                    .character_specs
                    .skills_specs
                    .iter()
                    .flat_map(|skill_specs| skill_specs.triggers.iter()),
            )
            .map(|trigger_specs| trigger_specs.triggered_effect.clone()),
    );

    // Apply modifiers on triggers that did not inherit modifiers from skill
    for trigger_specs in monster_specs
        .character_specs
        .triggers
        .iter_mut()
        .filter(|trigger_specs| !trigger_specs.inherit_modifiers)
    {
        for trigger_effect in trigger_specs.effects.iter_mut() {
            skills_updater::compute_skill_specs_effect(
                &trigger_specs.trigger_id,
                trigger_specs.skill_type,
                trigger_effect,
                effects.iter(),
            );
        }
    }
}
