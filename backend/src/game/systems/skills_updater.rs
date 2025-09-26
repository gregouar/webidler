use std::time::Duration;

use shared::data::{
    character_status::StatusSpecs,
    player::PlayerInventory,
    skill::{
        ItemStatsSource, ModifierEffectSource, SkillEffect, SkillEffectType, SkillSpecs,
        SkillState, SkillType,
    },
    stat_effect::{ApplyStatModifier, LuckyRollType, Modifier, StatEffect, StatType},
};

use crate::game::utils::rng::Rollable;

pub fn update_skills_states(
    elapsed_time: Duration,
    skill_specs: &[SkillSpecs],
    skill_states: &mut [SkillState],
) {
    for (skill_specs, skill_state) in skill_specs.iter().zip(skill_states.iter_mut()) {
        if skill_specs.cooldown > 0.0 {
            skill_state.elapsed_cooldown += elapsed_time.as_secs_f32() / skill_specs.cooldown;
        }
        if skill_state.elapsed_cooldown >= 1.0 {
            skill_state.elapsed_cooldown = 1.0;
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

pub fn update_skill_specs<'a>(
    skill_specs: &mut SkillSpecs,
    effects: impl Iterator<Item = &'a StatEffect> + Clone,
    inventory: Option<&PlayerInventory>,
) {
    skill_specs.targets = skill_specs.base.targets.clone();
    skill_specs.triggers = skill_specs.base.triggers.clone();
    skill_specs.cooldown = skill_specs.base.cooldown;
    skill_specs.mana_cost = skill_specs.base.mana_cost;

    // TODO: Could we do something better?
    let default_inventory = PlayerInventory::default();
    let inventory = inventory.unwrap_or(&default_inventory);

    let mut all_effects: Vec<_> = effects
        .cloned()
        .chain(compute_skill_upgrade_effects(
            skill_specs,
            skill_specs.upgrade_level,
        ))
        .chain(compute_skill_modifier_effects(skill_specs, inventory))
        .collect();

    all_effects.sort_by_key(|e| match e.modifier {
        Modifier::Flat => 0,
        Modifier::Multiplier => 1,
    });

    apply_effects_to_skill_specs(skill_specs, all_effects.iter());
}

pub fn apply_effects_to_skill_specs<'a>(
    skill_specs: &mut SkillSpecs,
    effects: impl Iterator<Item = &'a StatEffect> + Clone,
) {
    for effect in effects.clone() {
        if effect
            .stat
            .is_match(&StatType::Speed(Some(skill_specs.base.skill_type)))
        {
            skill_specs.cooldown.apply_negative_effect(effect);
        }
    }

    for skill_effect in skill_specs
        .targets
        .iter_mut()
        .flat_map(|t| t.effects.iter_mut())
        .chain(
            skill_specs
                .triggers
                .iter_mut()
                .flat_map(|trigger| trigger.triggered_effect.effects.iter_mut()),
        )
    {
        compute_skill_specs_effect(skill_specs.base.skill_type, skill_effect, effects.clone())
    }
}

pub fn compute_skill_upgrade_effects(
    skill_specs: &SkillSpecs,
    level: u16,
) -> impl Iterator<Item = StatEffect> + use<'_> + Clone {
    let level = level as f64 - 1.0;
    skill_specs
        .base
        .upgrade_effects
        .iter()
        .map(move |effect| StatEffect {
            stat: effect.stat,
            modifier: effect.modifier,
            value: match effect.modifier {
                Modifier::Multiplier => (1.0 + effect.value).powf(level) - 1.0,
                Modifier::Flat => effect.value * level,
            },
            bypass_ignore: true,
        })
}

fn compute_skill_modifier_effects<'a>(
    skill_specs: &'a SkillSpecs,
    inventory: &'a PlayerInventory,
) -> impl Iterator<Item = StatEffect> + use<'a> + Clone {
    skill_specs
        .base
        .modifier_effects
        .iter()
        .flat_map(move |modifier_effect| match modifier_effect.source {
            ModifierEffectSource::ItemStats { slot, item_stats } => inventory
                // .unwrap_or(&PlayerInventory::default())
                .equipped_items()
                .filter_map(move |(item_slot, item_specs)| {
                    let factor = if slot.unwrap_or(item_slot) == item_slot {
                        match (
                            item_stats,
                            &item_specs.weapon_specs,
                            &item_specs.armor_specs,
                        ) {
                            (ItemStatsSource::Damage(damage_type), Some(weapon_specs), _) => {
                                if let Some(damage_type) = damage_type {
                                    weapon_specs
                                        .damage
                                        .get(&damage_type)
                                        .map(|value| (value.min + value.max) * 0.5)
                                        .unwrap_or_default()
                                } else {
                                    weapon_specs
                                        .damage
                                        .values()
                                        .map(|value| (value.min + value.max) * 0.5)
                                        .sum()
                                }
                            }
                            (ItemStatsSource::Armor, _, Some(armor_specs)) => armor_specs.armor,
                            _ => 0.0,
                        }
                    } else {
                        0.0
                    };

                    if factor > 0.0 {
                        Some(modifier_effect.factor * factor)
                    } else {
                        None
                    }
                })
                .flat_map(|factor| {
                    modifier_effect
                        .effects
                        .iter()
                        .map(move |effect| StatEffect {
                            stat: effect.stat,
                            modifier: effect.modifier,
                            value: effect.value * factor,
                            bypass_ignore: true,
                        })
                }),
        })
}

pub fn compute_skill_specs_effect<'a>(
    skill_type: SkillType,
    skill_effect: &mut SkillEffect,
    effects: impl Iterator<Item = &'a StatEffect> + Clone,
) {
    if let SkillEffectType::ApplyStatus { statuses, .. } = &mut skill_effect.effect_type {
        for status_effect in statuses.iter_mut() {
            if let StatusSpecs::Trigger(ref mut trigger_specs) = status_effect.status_type {
                for triggered_effect in trigger_specs.triggered_effect.effects.iter_mut() {
                    compute_skill_specs_effect(skill_type, triggered_effect, effects.clone())
                }
            }
        }
    }

    for effect in effects.clone() {
        if !effect.bypass_ignore
            && skill_effect
                .ignore_stat_effects
                .iter()
                .any(|ignore| effect.stat.is_match(ignore))
        {
            continue;
        }

        if effect.stat.is_match(&StatType::Lucky {
            skill_type: Some(skill_type),
            roll_type: LuckyRollType::SuccessChance,
        }) {
            skill_effect
                .failure_chance
                .lucky_chance
                .apply_negative_effect(effect);
        }

        match &mut skill_effect.effect_type {
            SkillEffectType::FlatDamage {
                damage,
                crit_chance,
                crit_damage,
            } => {
                for (&damage_type, value) in damage.iter_mut() {
                    if effect.stat.is_match(&StatType::MinDamage {
                        skill_type: Some(skill_type),
                        damage_type: Some(damage_type),
                    }) || effect.stat.is_match(&StatType::Damage {
                        skill_type: Some(skill_type),
                        damage_type: Some(damage_type),
                    }) {
                        value.min.apply_effect(effect);
                    }

                    if effect.stat.is_match(&StatType::MaxDamage {
                        skill_type: Some(skill_type),
                        damage_type: Some(damage_type),
                    }) || effect.stat.is_match(&StatType::Damage {
                        skill_type: Some(skill_type),
                        damage_type: Some(damage_type),
                    }) {
                        value.max.apply_effect(effect);
                    }

                    if effect.stat.is_match(&StatType::Lucky {
                        skill_type: Some(skill_type),
                        roll_type: LuckyRollType::Damage {
                            damage_type: Some(damage_type),
                        },
                    }) {
                        value.lucky_chance.apply_effect(effect);
                    }
                }

                if effect
                    .stat
                    .is_match(&StatType::CritChance(Some(skill_type)))
                {
                    crit_chance.value.apply_effect(effect);
                }

                if effect
                    .stat
                    .is_match(&StatType::CritDamage(Some(skill_type)))
                {
                    crit_damage.apply_effect(effect);
                }

                crit_chance.clamp();
                damage.retain(|_, value| {
                    value.min = value.min.max(0.0);
                    value.max = value.max.max(0.0);
                    value.clamp();

                    value.max > 0.0
                });
            }
            SkillEffectType::ApplyStatus { statuses, duration } => {
                if statuses.iter().any(|status_effect| {
                    effect.stat.is_match(&StatType::StatusDuration(
                        (&status_effect.status_type).into(),
                    ))
                }) {
                    duration.min.apply_effect(effect);
                    duration.max.apply_effect(effect);
                }

                for status_effect in statuses.iter_mut() {
                    if effect
                        .stat
                        .is_match(&StatType::StatusPower((&status_effect.status_type).into()))
                    {
                        let effect = match (&status_effect.status_type, effect.modifier) {
                            // Correct because flat is in percent but multiplier in decimals
                            (
                                StatusSpecs::StatModifier {
                                    modifier: Modifier::Multiplier,
                                    ..
                                },
                                Modifier::Flat,
                            ) => StatEffect {
                                value: effect.value * 0.01,
                                ..*effect
                            },
                            _ => effect.clone(),
                        };

                        status_effect.value.min.apply_effect(&effect);
                        status_effect.value.max.apply_effect(&effect);
                    }

                    if let StatusSpecs::DamageOverTime { damage_type, .. } =
                        status_effect.status_type
                    {
                        if effect.stat.is_match(&StatType::MinDamage {
                            skill_type: Some(skill_type),
                            damage_type: Some(damage_type),
                        }) || effect.stat.is_match(&StatType::Damage {
                            skill_type: Some(skill_type),
                            damage_type: Some(damage_type),
                        }) {
                            status_effect.value.min.apply_effect(effect);
                        }

                        if effect.stat.is_match(&StatType::MaxDamage {
                            skill_type: Some(skill_type),
                            damage_type: Some(damage_type),
                        }) || effect.stat.is_match(&StatType::Damage {
                            skill_type: Some(skill_type),
                            damage_type: Some(damage_type),
                        }) {
                            status_effect.value.max.apply_effect(effect);
                        }

                        if effect.stat.is_match(&StatType::Lucky {
                            skill_type: Some(skill_type),
                            roll_type: LuckyRollType::Damage {
                                damage_type: Some(damage_type),
                            },
                        }) {
                            status_effect.value.lucky_chance.apply_effect(effect);
                        }
                    }
                }
            }
            SkillEffectType::Restore {
                restore_type,
                value,
                ..
            } => {
                if effect
                    .stat
                    .is_match(&StatType::Restore(Some(*restore_type)))
                {
                    value.min.apply_effect(effect);
                    value.max.apply_effect(effect);
                };
            }
            SkillEffectType::Resurrect => {}
        }
    }
}
