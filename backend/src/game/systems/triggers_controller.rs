use shared::data::{
    character::CharacterId,
    skill::SkillType,
    stat_effect::{StatEffect, StatStatusType, compare_options},
    trigger::{TriggerEffectModifierSource, TriggerTarget, TriggeredEffect},
    values::NonNegative,
};

use crate::game::{
    data::event::{EventsQueue, HitEvent, StatusEvent},
    game_data::GameInstanceData,
};

use super::{skills_controller, skills_updater};

#[derive(Debug)]
pub struct TriggerContext<'a> {
    pub trigger: TriggeredEffect,
    pub trigger_depth: u8,

    pub owner: CharacterId,
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
    if trigger_contexts.len() > 100 {
        tracing::error!("Too many triggers: {}", trigger_contexts.len());
    }

    for trigger_context in trigger_contexts.into_iter().take(100) {
        if trigger_context.trigger_depth > 3 {
            tracing::error!(
                "Trigger reached max depth: {:?}",
                trigger_context.trigger.trigger_id
            );
            continue;
        }

        let (target_id, attacker) = match trigger_context.trigger.target {
            TriggerTarget::SameTarget => (trigger_context.target, trigger_context.source),
            TriggerTarget::Source => (trigger_context.source, trigger_context.target),
            TriggerTarget::Me => (
                trigger_context.trigger.owner.unwrap_or(CharacterId::Player),
                trigger_context.trigger.owner.unwrap_or(CharacterId::Player),
            ),
        };

        // TODO: I don't think these should be StatusEvents...
        let statuses_context: Vec<StatusModifierData> =
            if let Some(status_context) = trigger_context.status_context {
                [StatusModifierData {
                    status_type: status_context.status_type.clone(),
                    skill_type: status_context.skill_type,
                    value: status_context.value,
                    duration: status_context.duration,
                }]
                .into()
            } else {
                match trigger_context.target {
                    CharacterId::Player => game_data
                        .player_state
                        .character_state
                        .statuses
                        .iter()
                        .map(|(status_specs, status_state)| StatusModifierData {
                            skill_type: status_state.skill_type,
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
                                .map(|(status_specs, status_state)| StatusModifierData {
                                    skill_type: status_state.skill_type,
                                    status_type: status_specs.into(),
                                    value: status_state.value,
                                    duration: status_state.duration,
                                })
                                .collect()
                        })
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
                            (
                                m.character_specs.character_static.position_x,
                                m.character_specs.character_static.position_y,
                            ),
                            m.character_specs.character_static.size.get_xy_size(),
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

        let trigger_effects: Vec<_> = if trigger_context.trigger.modifiers.is_empty() {
            trigger_context.trigger.effects
        } else {
            let source_effects: Vec<_> = trigger_context
                .trigger
                .modifiers
                .iter()
                .map(|modifier| StatEffect {
                    stat: modifier.stat.clone(),
                    modifier: modifier.modifier,
                    value: modifier.factor
                        * match &modifier.source {
                            TriggerEffectModifierSource::HitDamage(Some(damage_type)) => {
                                trigger_context
                                    .hit_context
                                    .as_ref()
                                    .and_then(|hit| hit.damage.get(damage_type))
                                    .map(|d| d.get())
                                    .unwrap_or_default()
                            }
                            TriggerEffectModifierSource::HitDamage(None) => trigger_context
                                .hit_context
                                .as_ref()
                                .map(|hit| hit.damage.values().map(|d| d.get()).sum())
                                .unwrap_or_default(),
                            TriggerEffectModifierSource::HitCrit => trigger_context
                                .hit_context
                                .as_ref()
                                .map(|hit| hit.is_crit as i64 as f64)
                                .unwrap_or_default(),
                            TriggerEffectModifierSource::AreaLevel => {
                                trigger_context.level as f64
                                    + *game_data.area_specs.power_level as f64
                            }
                            TriggerEffectModifierSource::StatusValue {
                                status_type,
                                skill_type,
                            } => statuses_context
                                .iter()
                                .filter(|status_data| {
                                    compare_options(
                                        &status_type.as_ref(),
                                        &Some(&status_data.status_type),
                                    ) && compare_options(
                                        &skill_type.as_ref(),
                                        &Some(&status_data.skill_type),
                                    )
                                })
                                .map(|status_event| status_event.value.get())
                                .sum(),
                            TriggerEffectModifierSource::StatusDuration {
                                status_type,
                                skill_type,
                            } => statuses_context
                                .iter()
                                .filter(|status_data| {
                                    compare_options(
                                        &status_type.as_ref(),
                                        &Some(&status_data.status_type),
                                    ) && compare_options(
                                        &skill_type.as_ref(),
                                        &Some(&status_data.skill_type),
                                    )
                                })
                                .map(|status_data| {
                                    status_data.duration.map(|d| d.get()).unwrap_or(1e20)
                                })
                                .sum(),
                            TriggerEffectModifierSource::StatusStacks {
                                status_type,
                                skill_type,
                            } => statuses_context
                                .iter()
                                .filter(|status_event| {
                                    compare_options(
                                        &status_type.as_ref(),
                                        &Some(&status_event.status_type),
                                    ) && compare_options(
                                        &skill_type.as_ref(),
                                        &Some(&status_event.skill_type),
                                    )
                                })
                                .count() as f64,
                            TriggerEffectModifierSource::TriggerStatusValue => 0.0,
                        },
                    bypass_ignore: true,
                })
                .collect();

            trigger_context
                .trigger
                .effects
                .into_iter()
                .map(|mut effect| {
                    skills_updater::compute_skill_specs_effect(
                        &trigger_context.trigger.trigger_id,
                        trigger_context.trigger.skill_type,
                        &mut effect,
                        source_effects.iter(),
                    );
                    effect
                })
                .collect()
        };

        skills_controller::apply_skill_effects(
            events_queue,
            attacker,
            &trigger_context.trigger.trigger_id,
            trigger_context.trigger.skill_type,
            trigger_context.trigger.skill_range,
            &trigger_effects,
            &mut targets,
            if trigger_context.trigger.trigger_propagate {
                0
            } else {
                trigger_context.trigger_depth.saturating_add(1)
            },
        );
    }
}

struct StatusModifierData {
    status_type: StatStatusType,
    skill_type: SkillType,
    value: NonNegative,
    duration: Option<NonNegative>,
}
