use std::time::Duration;

use shared::data::{
    character_status::StatusType,
    item_affix::{Modifier, StatType},
    passive::StatEffect,
    skill::{DamageType, SkillEffect, SkillEffectType, SkillSpecs, SkillState, SkillType},
    stat_effect::DamageMap,
};

use super::stats_controller::ApplyStatModifier;

pub fn update_skills_states(
    elapsed_time: Duration,
    skill_specs: &[SkillSpecs],
    skill_states: &mut [SkillState],
) {
    for (skill_specs, skill_state) in skill_specs.iter().zip(skill_states.iter_mut()) {
        skill_state.elapsed_cooldown += elapsed_time.as_secs_f32();
        if skill_state.elapsed_cooldown >= skill_specs.cooldown {
            skill_state.elapsed_cooldown = skill_specs.cooldown;
            skill_state.is_ready = true;
        } else {
            skill_state.is_ready = false;
        }
    }
}

pub fn reset_skills(skill_states: &mut [SkillState]) {
    for skill_state in skill_states.iter_mut() {
        skill_state.just_triggered = false;
    }
}

pub fn update_skill_specs(skill_specs: &mut SkillSpecs, effects: &[StatEffect]) {
    skill_specs.targets = skill_specs.base.targets.clone();
    skill_specs.cooldown = skill_specs.base.cooldown;
    skill_specs.mana_cost = skill_specs.base.mana_cost;

    let base_effects = compute_skill_upgrade_effects(&skill_specs, skill_specs.upgrade_level);

    let effects = effects.iter().chain(base_effects.iter());

    for effect in effects.clone() {
        match effect.stat {
            StatType::Speed(skill_type)
                if skill_specs.base.skill_type
                    == skill_type.unwrap_or(skill_specs.base.skill_type) =>
            {
                skill_specs.cooldown.apply_inverse_effect(effect);
            }
            _ => {}
        }
    }

    for skill_effect in skill_specs
        .targets
        .iter_mut()
        .flat_map(|t| t.effects.iter_mut())
    {
        compute_skill_specs_effect(skill_specs.base.skill_type, skill_effect, effects.clone())
    }
}

pub fn compute_skill_upgrade_effects(skill_specs: &SkillSpecs, level: u16) -> Vec<StatEffect> {
    let level = level as f64 - 1.0;
    skill_specs
        .base
        .upgrade_effects
        .iter()
        .map(|effect| StatEffect {
            stat: effect.stat,
            modifier: effect.modifier,
            value: match effect.modifier {
                Modifier::Multiplier => (1.0 + effect.value).powf(level) - 1.0,
                Modifier::Flat => effect.value * level,
            },
        })
        .collect::<Vec<_>>()
}

pub fn compute_skill_specs_effect<'a, I>(
    skill_type: SkillType,
    skill_effect: &mut SkillEffect,
    effects: I,
) where
    I: IntoIterator<Item = &'a StatEffect>,
{
    for effect in effects {
        match &mut skill_effect.effect_type {
            SkillEffectType::FlatDamage {
                damage,
                crit_chances,
                crit_damage,
            } => {
                match effect.stat {
                    StatType::SpellPower if skill_type == SkillType::Spell => {
                        for (min, max) in damage.values_mut() {
                            min.apply_effect(effect);
                            max.apply_effect(effect);
                        }
                    }
                    StatType::Damage {
                        skill_type: skill_type2,
                        damage_type,
                    } if skill_type == skill_type2.unwrap_or(skill_type) => {
                        apply_effect_on_damage(damage, damage_type, Some(effect), Some(effect))
                    }
                    StatType::MinDamage {
                        skill_type: skill_type2,
                        damage_type,
                    } if skill_type == skill_type2.unwrap_or(skill_type) => {
                        apply_effect_on_damage(damage, damage_type, Some(effect), None)
                    }
                    StatType::MaxDamage {
                        skill_type: skill_type2,
                        damage_type,
                    } if skill_type == skill_type2.unwrap_or(skill_type) => {
                        apply_effect_on_damage(damage, damage_type, None, Some(effect))
                    }
                    _ => {}
                }
                match effect.stat {
                    StatType::CritChances(skill_type2)
                        if skill_type == skill_type2.unwrap_or(skill_type) =>
                    {
                        crit_chances.apply_effect(effect);
                    }
                    StatType::CritDamage(skill_type2)
                        if skill_type == skill_type2.unwrap_or(skill_type) =>
                    {
                        crit_damage.apply_effect(effect);
                    }
                    _ => {}
                }

                *crit_chances = crit_chances.clamp(0.0, 1.0);
                damage.retain(|_, (min, max)| {
                    *min = min.clamp(0.0, *max);
                    *max > 0.0
                });
            }
            SkillEffectType::Heal { min, max } => {
                if effect.stat == StatType::SpellPower {
                    min.apply_effect(effect);
                    max.apply_effect(effect);
                }
            }
            SkillEffectType::ApplyStatus {
                status_type,
                min_value,
                max_value,
                ..
            } => match status_type {
                StatusType::Stun => {
                    // Something?
                }
                StatusType::DamageOverTime { damage_type, .. } => match effect.stat {
                    StatType::SpellPower if skill_type == SkillType::Spell => {
                        min_value.apply_effect(effect);
                        max_value.apply_effect(effect);
                    }
                    StatType::Damage {
                        skill_type: skill_type2,
                        damage_type: damage_type2,
                    } if skill_type == skill_type2.unwrap_or(skill_type)
                        && *damage_type == damage_type2.unwrap_or(*damage_type) =>
                    {
                        min_value.apply_effect(effect);
                        max_value.apply_effect(effect);
                    }
                    _ => {}
                },
                StatusType::StatModifier(_) => {
                    if StatType::SpellPower == effect.stat && skill_type == SkillType::Spell {
                        min_value.apply_effect(effect);
                        max_value.apply_effect(effect);
                    }
                }
            },
        }
    }
}

fn apply_effect_on_damage(
    damage: &mut DamageMap,
    damage_type: Option<DamageType>,
    min_effect: Option<&StatEffect>,
    max_effect: Option<&StatEffect>,
) {
    match damage_type {
        Some(damage_type) => {
            let (min, max) = damage.entry(damage_type).or_insert((0.0, 0.0));
            if let Some(e) = min_effect {
                min.apply_effect(e);
            }
            if let Some(e) = max_effect {
                max.apply_effect(e);
            }
        }
        None => {
            for (min, max) in damage.values_mut() {
                if let Some(e) = min_effect {
                    min.apply_effect(e);
                }
                if let Some(e) = max_effect {
                    max.apply_effect(e);
                }
            }
        }
    }
}
