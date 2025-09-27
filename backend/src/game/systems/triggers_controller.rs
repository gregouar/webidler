use shared::data::{
    character::CharacterId,
    passive::StatEffect,
    trigger::{TriggerEffectModifierSource, TriggerTarget, TriggeredEffect},
};

use crate::game::{
    data::event::{EventsQueue, HitEvent},
    game_data::GameInstanceData,
};

use super::{skills_controller, skills_updater};

pub struct TriggerContext<'a> {
    pub trigger: TriggeredEffect,

    pub owner: CharacterId,
    pub source: CharacterId,
    pub target: CharacterId,
    pub hit_context: Option<&'a HitEvent>,
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
                TriggerTarget::Me => (trigger_context.owner, trigger_context.owner),
            };

            // TODO: Only clone ids and values
            let statuses_context = match trigger_context.target {
                CharacterId::Player => game_data.player_state.character_state.statuses.clone(),
                CharacterId::Monster(index) => game_data
                    .monster_states
                    .get(index)
                    .map(|monster_state| monster_state.character_state.statuses.clone())
                    .unwrap_or_default(),
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
                    let target_position = game_data
                        .monster_specs
                        .get(i)
                        .map(|m| (m.character_specs.position_x, m.character_specs.position_y))
                        .unwrap_or_default();
                    skills_controller::find_sub_targets(
                        trigger_context.trigger.skill_range,
                        trigger_context.trigger.skill_shape,
                        target_position,
                        &mut monsters_still_alive,
                    )
                }
            };

            let effects_modifiers: Vec<_> = trigger_context
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
                            TriggerEffectModifierSource::AreaLevel => trigger_context.level as f64,
                            TriggerEffectModifierSource::StatusValue(stat_status_type) => {
                                statuses_context
                                    .iter()
                                    .filter(|(status_specs, _)| match stat_status_type {
                                        Some(stat_status_type) => {
                                            if let Some(status_type) = status_specs.into() {
                                                stat_status_type.is_match(&status_type)
                                            } else {
                                                false
                                            }
                                        }
                                        None => true,
                                    })
                                    .map(|(_, status_state)| status_state.value)
                                    .sum()
                            }
                            TriggerEffectModifierSource::StatusDuration(stat_status_type) => {
                                statuses_context
                                    .iter()
                                    .filter(|(status_specs, _)| match stat_status_type {
                                        Some(stat_status_type) => {
                                            if let Some(status_type) = status_specs.into() {
                                                stat_status_type.is_match(&status_type)
                                            } else {
                                                false
                                            }
                                        }
                                        None => true,
                                    })
                                    .map(|(_, status_state)| {
                                        status_state.duration.unwrap_or_default()
                                    })
                                    .sum()
                            }
                        },
                    bypass_ignore: true,
                })
                .collect();

            for mut effect in trigger_context.trigger.effects.iter().cloned() {
                skills_updater::compute_skill_specs_effect(
                    trigger_context.trigger.skill_type,
                    &mut effect,
                    effects_modifiers.iter(),
                );
                skills_controller::apply_skill_effect(
                    events_queue,
                    attacker,
                    trigger_context.trigger.skill_type,
                    trigger_context.trigger.skill_range,
                    &effect,
                    &mut targets,
                );
            }
        }
    }
}
