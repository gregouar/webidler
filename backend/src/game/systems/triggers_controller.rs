use shared::data::{
    area::AreaLevel,
    character::CharacterId,
    passive::StatEffect,
    trigger::{TriggerEffectModifierSource, TriggerTarget, TriggeredEffect},
};

use crate::game::{
    data::{
        event::{EventsQueue, HitEvent},
        master_store::MasterStore,
    },
    game_data::GameInstanceData,
};

use super::{skills_controller, skills_updater};

pub struct TriggerContext<'a> {
    pub trigger: TriggeredEffect,

    pub source: CharacterId,
    pub target: CharacterId,
    pub hit_context: Option<&'a HitEvent>,
    pub area_level: AreaLevel,
}

pub fn apply_trigger_effects(
    events_queue: &mut EventsQueue,
    game_data: &mut GameInstanceData,
    master_store: &MasterStore,
    trigger_contexts: Vec<TriggerContext>,
) {
    let _ = master_store; // TODO: Remove
    for trigger_context in trigger_contexts {
        {
            let target_id = match trigger_context.trigger.target {
                TriggerTarget::SameTarget => trigger_context.target,
                TriggerTarget::Source => trigger_context.source,
            };
            // TODO: Multi targets
            let target = match target_id {
                CharacterId::Player => (
                    Some(&game_data.player_specs.read().character_specs),
                    Some(&mut game_data.player_state.character_state),
                ),
                CharacterId::Monster(i) => (
                    game_data.monster_specs.get(i).map(|m| &m.character_specs),
                    game_data
                        .monster_states
                        .get_mut(i)
                        .map(|m| &mut m.character_state),
                ),
            };

            let effects_modifiers: Vec<_> = trigger_context
                .trigger
                .modifiers
                .iter()
                .map(|modifier| StatEffect {
                    stat: modifier.stat,
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
                            TriggerEffectModifierSource::AreaLevel => {
                                trigger_context.area_level as f64
                            }
                        },
                })
                .collect();

            if let (Some(specs), Some(state)) = target {
                let mut target = (target_id, (specs, state));
                let mut targets = vec![&mut target];
                for mut effect in trigger_context.trigger.effects.iter().cloned() {
                    skills_updater::compute_skill_specs_effect(
                        trigger_context.trigger.skill_type,
                        &mut effect,
                        effects_modifiers.iter(),
                    );
                    skills_controller::apply_skill_effect(
                        events_queue,
                        trigger_context.source,
                        trigger_context.trigger.skill_type,
                        trigger_context.trigger.skill_range,
                        &effect,
                        &mut targets,
                    );
                }
            }
        }
    }
}
