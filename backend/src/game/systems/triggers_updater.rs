use shared::data::{
    stat_effect::{StatEffect, StatType},
    trigger::{TriggerEffect, TriggerEffectModifier},
};

use crate::game::{data::master_store::StatusesStore, systems::skills_updater};

pub fn compute_trigger_specs_effects<'a>(
    statuses_store: &StatusesStore,
    trigger_effect: &mut TriggerEffect,
    effects: impl Iterator<Item = &'a StatEffect> + Clone,
) {
    compute_trigger_specs_effects_with_extra(
        statuses_store,
        trigger_effect,
        effects,
        std::iter::empty(),
    );
}

pub fn compute_trigger_specs_effects_with_extra<'a, 'b>(
    statuses_store: &StatusesStore,
    trigger_effect: &mut TriggerEffect,
    effects: impl Iterator<Item = &'a StatEffect> + Clone,
    extra_effects: impl Iterator<Item = &'b StatEffect> + Clone,
) {
    apply_trigger_effect_modifiers(trigger_effect, effects.clone());
    apply_trigger_effect_modifiers(trigger_effect, extra_effects.clone());

    for skill_effect in trigger_effect.effects.iter_mut() {
        skills_updater::compute_skill_specs_effect_with_extra(
            statuses_store,
            &trigger_effect.trigger_id,
            trigger_effect.skill_type,
            skill_effect,
            effects.clone(),
            extra_effects.clone(),
        );
    }
}

fn apply_trigger_effect_modifiers<'a>(
    trigger_effect: &mut TriggerEffect,
    effects: impl Iterator<Item = &'a StatEffect>,
) {
    for effect in effects {
        if let StatType::TriggerEffectModifier {
            stat,
            source,
            skill_filter,
        } = &effect.stat
            && skill_filter
                .is_match_with_skill(trigger_effect.skill_type, &trigger_effect.trigger_id)
        {
            trigger_effect.modifiers.push(TriggerEffectModifier {
                stat: *stat.clone(),
                modifier: effect.modifier,
                factor: effect.value,
                source: source.clone(),
            });
        }
    }
}
