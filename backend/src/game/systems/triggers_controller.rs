use shared::data::{
    character::CharacterId,
    passive::StatEffect,
    trigger::{TriggerEffectModifierSource, TriggerTarget, TriggeredEffect},
};

use crate::game::{
    data::event::{EventsQueue, HitEvent, StatusEvent},
    game_data::GameInstanceData,
    systems::stats_updater,
};

use super::{skills_controller, skills_updater};

pub struct TriggerContext<'a> {
    pub trigger: TriggeredEffect,

    pub source: CharacterId,
    pub target: CharacterId,
    pub hit_context: Option<&'a HitEvent>,
    pub status_context: Option<&'a StatusEvent>,
    pub level: usize,
}

pub fn apply_trigger_effects(
    events_queue: &mut EventsQueue,
    game_data: &mut GameInstanceData,
    trigger_contexts: Vec<TriggerContext>,
) {
    for trigger_context in trigger_contexts {
        {
            let (target_id, attacker) = match trigger_context.trigger.target {
                TriggerTarget::SameTarget => (trigger_context.target, trigger_context.source),
                TriggerTarget::Source => (trigger_context.source, trigger_context.target),
                TriggerTarget::Me => (
                    trigger_context.trigger.owner.unwrap_or(CharacterId::Player),
                    trigger_context.trigger.owner.unwrap_or(CharacterId::Player),
                ),
            };

            let statuses_context: Vec<_> =
                if let Some(status_context) = trigger_context.status_context {
                    [status_context.clone()].into()
                } else {
                    match trigger_context.target {
                        CharacterId::Player => game_data
                            .player_state
                            .character_state
                            .statuses
                            .iter()
                            .map(|(status_specs, status_state)| StatusEvent {
                                source: trigger_context.source,
                                target: trigger_context.target,
                                skill_type: status_state.skill_type,
                                is_triggered: false,
                                status_type: status_specs.into(),
                                value: status_state.value,
                                duration: status_state.duration,
                            })
                            .collect(),
                        CharacterId::Monster(index) => game_data
                            .monster_states
                            .get(index)
                            .map(|monster_state| {
                                monster_state
                                    .character_state
                                    .statuses
                                    .iter()
                                    .map(|(status_specs, status_state)| StatusEvent {
                                        source: trigger_context.source,
                                        target: trigger_context.target,
                                        skill_type: status_state.skill_type,
                                        is_triggered: false,
                                        status_type: status_specs.into(),
                                        value: status_state.value,
                                        duration: status_state.duration,
                                    })
                                    .collect()
                            })
                            .unwrap_or_default(),
                    }
                };

            let mut source_effects: Vec<_> = if trigger_context.trigger.inherit_modifiers {
                Vec::new()
            } else {
                match trigger_context.trigger.owner.unwrap_or(CharacterId::Player) {
                    CharacterId::Player => {
                        (&game_data.player_specs.read().character_specs.effects).into()
                    }
                    CharacterId::Monster(index) => game_data
                        .monster_specs
                        .get(index)
                        .map(|monster_specs| (&monster_specs.character_specs.effects).into())
                        .unwrap_or_default(),
                }
            };

            let mut player_target = (
                CharacterId::Player,
                (
                    &game_data.player_specs.read().character_specs,
                    &mut game_data.player_state.character_state,
                ),
            );

            let mut monsters_still_alive: Vec<_> = game_data
                .monster_specs
                .iter()
                .zip(game_data.monster_states.iter_mut())
                .enumerate()
                .filter(|(_, (_, m))| m.character_state.is_alive)
                .map(|(i, (x, y))| {
                    (
                        CharacterId::Monster(i),
                        (&x.character_specs, &mut y.character_state),
                    )
                })
                .collect();

            let mut targets = match target_id {
                CharacterId::Player => {
                    vec![&mut player_target]
                }
                CharacterId::Monster(i) => {
                    let (target_position, target_size) = game_data
                        .monster_specs
                        .get(i)
                        .map(|m| {
                            (
                                (m.character_specs.position_x, m.character_specs.position_y),
                                m.character_specs.size.get_xy_size(),
                            )
                        })
                        .unwrap_or_default();
                    skills_controller::find_sub_targets(
                        trigger_context.trigger.skill_range,
                        trigger_context.trigger.skill_shape,
                        target_position,
                        target_size,
                        &mut monsters_still_alive,
                    )
                }
            };

            source_effects.extend(
                // let mut effects_modifiers: Vec<_> =
                trigger_context
                    .trigger
                    .modifiers
                    .iter()
                    .map(|modifier| StatEffect {
                        stat: modifier.stat.clone(),
                        modifier: modifier.modifier,
                        value: modifier.factor
                            * match modifier.source {
                                TriggerEffectModifierSource::HitDamage(Some(damage_type)) => {
                                    trigger_context
                                        .hit_context
                                        .as_ref()
                                        .and_then(|hit| hit.damage.get(&damage_type))
                                        .copied()
                                        .unwrap_or_default()
                                }
                                TriggerEffectModifierSource::HitDamage(None) => trigger_context
                                    .hit_context
                                    .as_ref()
                                    .map(|hit| hit.damage.values().sum())
                                    .unwrap_or_default(),
                                TriggerEffectModifierSource::HitCrit => trigger_context
                                    .hit_context
                                    .as_ref()
                                    .map(|hit| hit.is_crit as i64 as f64)
                                    .unwrap_or_default(),
                                TriggerEffectModifierSource::AreaLevel => {
                                    trigger_context.level as f64
                                }
                                TriggerEffectModifierSource::StatusValue(stat_status_type) => {
                                    statuses_context
                                        .iter()
                                        .filter(|status_event| match stat_status_type {
                                            Some(stat_status_type) => {
                                                stat_status_type.is_match(&status_event.status_type)
                                            }
                                            None => true,
                                        })
                                        .map(|status_event| status_event.value)
                                        .sum()
                                }
                                TriggerEffectModifierSource::StatusDuration(stat_status_type) => {
                                    statuses_context
                                        .iter()
                                        .filter(|status_event| match stat_status_type {
                                            Some(stat_status_type) => {
                                                stat_status_type.is_match(&status_event.status_type)
                                            }
                                            None => true,
                                        })
                                        .map(|status_event| status_event.duration.unwrap_or(1e20))
                                        .sum()
                                }
                                TriggerEffectModifierSource::StatusStacks(stat_status_type) => {
                                    statuses_context
                                        .iter()
                                        .filter(|status_event| match stat_status_type {
                                            Some(stat_status_type) => {
                                                stat_status_type.is_match(&status_event.status_type)
                                            }
                                            None => true,
                                        })
                                        .count() as f64
                                }
                            },
                        bypass_ignore: true,
                    }),
            );
            // .collect();

            let stat_effects =
                stats_updater::combine_effects(source_effects, &game_data.area_threat);

            // stats_updater::sort_stat_effects(&mut effects_modifiers);

            for mut effect in trigger_context.trigger.effects.iter().cloned() {
                skills_updater::compute_skill_specs_effect(
                    trigger_context.trigger.skill_type,
                    &mut effect,
                    stat_effects.iter(),
                );
                skills_controller::apply_skill_effect(
                    events_queue,
                    attacker,
                    trigger_context.trigger.skill_type,
                    trigger_context.trigger.skill_range,
                    &effect,
                    &mut targets,
                    true,
                );
            }
        }
    }
}
