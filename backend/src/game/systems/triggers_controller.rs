use shared::data::{
    character::CharacterId,
    character_status::StatusId,
    skill::{DamageType, SkillType},
    stat_effect::{StatEffect, compare_options},
    trigger::{OwnedTrigger, TriggerEffectModifierSource, TriggerTarget},
    values::NonNegative,
};

use crate::game::{
    data::{
        event::{EventsQueue, HitEvent, StatusEvent},
        master_store::StatusesStore,
    },
    game_data::GameInstanceData,
};

use super::{skills_controller, skills_updater};

#[derive(Debug)]
pub struct TriggerContext<'a> {
    pub owned_trigger: OwnedTrigger, //TODO: Replace by lazy?
    pub trigger_depth: u8,

    pub source: CharacterId,
    pub target: CharacterId,

    pub hit_context: Option<&'a HitEvent>,
    pub status_context: Option<&'a StatusEvent>,
    pub level: usize,
}

pub fn apply_trigger_effects(
    statuses_store: &StatusesStore,
    events_queue: &mut EventsQueue,
    game_data: &mut GameInstanceData,
    trigger_contexts: Vec<TriggerContext>,
) {
    if trigger_contexts.len() > 100 {
        tracing::error!("Too many triggers: {}", trigger_contexts.len());
    }

    for trigger_context in trigger_contexts.into_iter().take(100) {
        let trigger_effect = trigger_context.owned_trigger.trigger_effect;

        if trigger_context.trigger_depth > 3 {
            tracing::error!(
                "Trigger reached max depth: {:?}",
                &trigger_effect.trigger_id
            );
            continue;
        }

        let (target_id, attacker) = match trigger_effect.target {
            TriggerTarget::SameTarget => (trigger_context.target, trigger_context.source),
            TriggerTarget::Source => (trigger_context.source, trigger_context.target),
            TriggerTarget::Me => (
                trigger_context
                    .owned_trigger
                    .owner
                    .unwrap_or(CharacterId::Player),
                trigger_context
                    .owned_trigger
                    .owner
                    .unwrap_or(CharacterId::Player),
            ),
        };

        let statuses_context: Vec<StatusModifierData> =
            if let Some(status_context) = trigger_context.status_context {
                [StatusModifierData {
                    status_id: &status_context.status_id,
                    damage_type: status_context.damage_type,
                    skill_type: status_context.skill_type,
                    value: status_context.value,
                    duration: status_context.duration,
                }]
                .into()
            } else {
                game_data
                    .character_state(trigger_context.target)
                    .map(|character_state| {
                        character_state
                            .statuses
                            .iter()
                            .flat_map(|(status_id, status_stacks)| {
                                let damage_type = statuses_store
                                    .get(status_id)
                                    .and_then(|status_specs| status_specs.damage_type);
                                status_stacks
                                    .iter()
                                    .map(move |status_state| StatusModifierData {
                                        skill_type: status_state.skill_type,
                                        damage_type,
                                        status_id: status_id,
                                        value: status_state.value,
                                        duration: status_state.duration,
                                    })
                            })
                            .collect()
                    })
                    .unwrap_or_default()
            };

        let trigger_effects: Vec<_> =
            if trigger_effect.modifiers.is_empty() && !trigger_effect.inherit_source_effects {
                trigger_effect.effects
            } else {
                let modifier_effects: Vec<_> = trigger_effect
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
                                    status_filter,
                                    skill_type,
                                } => statuses_context
                                    .iter()
                                    .filter(|status_data| {
                                        status_filter.is_match_with_status(
                                            status_data.status_id,
                                            status_data.damage_type,
                                        ) && compare_options(
                                            &skill_type.as_ref(),
                                            &Some(&status_data.skill_type),
                                        )
                                    })
                                    .map(|status_event| status_event.value.get())
                                    .sum(),
                                TriggerEffectModifierSource::StatusDuration {
                                    status_filter,
                                    skill_type,
                                } => statuses_context
                                    .iter()
                                    .filter(|status_data| {
                                        status_filter.is_match_with_status(
                                            status_data.status_id,
                                            status_data.damage_type,
                                        ) && compare_options(
                                            &skill_type.as_ref(),
                                            &Some(&status_data.skill_type),
                                        )
                                    })
                                    .map(|status_data| status_data.duration.get())
                                    .sum(),
                                TriggerEffectModifierSource::StatusStacks {
                                    status_filter,
                                    skill_type,
                                } => statuses_context
                                    .iter()
                                    .filter(|status_data| {
                                        status_filter.is_match_with_status(
                                            status_data.status_id,
                                            status_data.damage_type,
                                        ) && compare_options(
                                            &skill_type.as_ref(),
                                            &Some(&status_data.skill_type),
                                        )
                                    })
                                    .count() as f64,
                                TriggerEffectModifierSource::TriggerStatusDuration => 0.0,
                                TriggerEffectModifierSource::TriggerStatusValue => 0.0,
                            },
                        bypass_ignore: true,
                    })
                    .collect();

                let source_effects: Vec<_> = if trigger_effect.inherit_source_effects {
                    game_data
                        .character_specs(trigger_context.source)
                        .map(|character_specs| character_specs.effects.iter().collect())
                        .unwrap_or_default()
                } else {
                    Default::default()
                };

                trigger_effect
                    .effects
                    .into_iter()
                    .map(|mut effect| {
                        skills_updater::compute_skill_specs_effect(
                            statuses_store,
                            &trigger_effect.trigger_id,
                            trigger_effect.skill_type,
                            &mut effect,
                            modifier_effects.iter().chain(source_effects.iter()),
                        );
                        effect
                    })
                    .collect()
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
                    trigger_effect.skill_range,
                    trigger_effect.skill_shape,
                    target_position,
                    target_size,
                    &mut monsters_still_alive,
                )
            }
        };

        skills_controller::apply_skill_effects(
            statuses_store,
            events_queue,
            attacker,
            &trigger_effect.trigger_id,
            trigger_effect.skill_type,
            trigger_effect.skill_range,
            &trigger_effects,
            &mut targets,
            if trigger_effect.trigger_propagate {
                0
            } else {
                trigger_context.trigger_depth.saturating_add(1)
            },
        );
    }
}

struct StatusModifierData<'a> {
    status_id: &'a StatusId,
    damage_type: Option<DamageType>,
    skill_type: SkillType,
    value: NonNegative,
    duration: NonNegative,
}
