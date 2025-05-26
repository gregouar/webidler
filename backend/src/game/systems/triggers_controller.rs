use shared::data::{
    character::CharacterId,
    item::SkillRange,
    passive::StatEffect,
    skill::SkillType,
    trigger::{TriggerEffectModifierSource, TriggerEffectType, TriggerTarget},
    world::AreaLevel,
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
    pub effect: TriggerEffectType,
    pub source: CharacterId,
    pub target: CharacterId,
    pub hit_context: Option<&'a HitEvent>,
    pub area_level: AreaLevel,
}

pub fn apply_trigger_effects(
    events_queue: &mut EventsQueue,
    game_data: &mut GameInstanceData,
    master_store: &MasterStore,
    trigger_effects: Vec<TriggerContext>,
) {
    let _ = master_store; // TODO: Remove
    for trigger_effect in trigger_effects {
        match trigger_effect.effect {
            TriggerEffectType::UseSkill => todo!(),
            TriggerEffectType::ApplySkillEffects {
                target,
                effects,
                modifiers,
            } => {
                let target_id = match target {
                    TriggerTarget::SameTarget => trigger_effect.target,
                    TriggerTarget::Source => trigger_effect.source,
                };
                // TODO: Multi targets
                let target = match target_id {
                    CharacterId::Player => (
                        Some(&game_data.player_specs.read().character_specs),
                        Some(&mut game_data.player_state.character_state),
                    ),
                    CharacterId::Monster(i) => (
                        game_data
                            .monster_specs
                            .read()
                            .get(i)
                            .map(|m| &m.character_specs),
                        game_data
                            .monster_states
                            .get_mut(i)
                            .map(|m| &mut m.character_state),
                    ),
                };

                let effects_modifiers: Vec<_> = modifiers
                    .iter()
                    .map(|modifier| StatEffect {
                        stat: modifier.stat,
                        modifier: modifier.modifier,
                        value: modifier.factor
                            * match modifier.source {
                                TriggerEffectModifierSource::HitDamage(Some(damage_type)) => {
                                    trigger_effect
                                        .hit_context
                                        .as_ref()
                                        .and_then(|hit| hit.damage.get(&damage_type))
                                        .map(|x| *x)
                                        .unwrap_or_default()
                                }
                                TriggerEffectModifierSource::HitDamage(None) => trigger_effect
                                    .hit_context
                                    .as_ref()
                                    .map(|hit| hit.damage.values().sum())
                                    .unwrap_or_default(),
                                TriggerEffectModifierSource::AreaLevel => {
                                    trigger_effect.area_level as f64
                                }
                            },
                    })
                    .collect();

                let skill_type = SkillType::Spell; // TODO
                if let (Some(specs), Some(state)) = target {
                    let mut target = (target_id, (specs, state));
                    let mut targets = vec![&mut target];
                    for mut effect in effects.iter().cloned() {
                        skills_updater::compute_skill_specs_effect(
                            skill_type,
                            &mut effect,
                            &effects_modifiers,
                        );
                        skills_controller::apply_skill_effect(
                            events_queue,
                            trigger_effect.source,
                            skill_type,
                            SkillRange::Any, // TODO
                            &effect,
                            &mut targets,
                        );
                    }
                }
            }
        }
    }
}
