use std::{collections::HashMap, time::Duration};

use shared::data::{
    area::AreaThreat,
    character_status::StatusSpecs,
    player::PlayerInventory,
    skill::{
        DamageType, ItemStatsSource, ModifierEffectSource, SkillEffect, SkillEffectType,
        SkillSpecs, SkillState, SkillType,
    },
    stat_effect::{
        ApplyStatModifier, EffectsMap, LuckyRollType, Modifier, StatConverterSource, StatEffect,
        StatType,
    },
};
use strum::IntoEnumIterator;

use crate::game::{systems::stats_updater, utils::rng::Rollable};

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
    area_threat: &AreaThreat,
) {
    skill_specs.targets = skill_specs.base.targets.clone();
    skill_specs.triggers = skill_specs.base.triggers.clone();
    skill_specs.cooldown = skill_specs.base.cooldown;
    skill_specs.mana_cost = skill_specs.base.mana_cost;

    // let mut local_effects: Vec<_> =
    //     compute_skill_upgrade_effects(skill_specs, skill_specs.upgrade_level)
    //         .chain(compute_skill_modifier_effects(skill_specs, inventory))
    //         .collect();

    let local_effects = stats_updater::stats_map_to_vec(
        &EffectsMap::combine_all(
            std::iter::once(compute_skill_upgrade_effects(
                skill_specs,
                skill_specs.upgrade_level,
            ))
            .chain(std::iter::once(compute_skill_modifier_effects(
                skill_specs,
                inventory,
            ))),
        ),
        area_threat,
    );

    // local_effects.sort_by_key(|e| match e.modifier {
    //     Modifier::Flat => 0,
    //     Modifier::Multiplier => 1,
    // });

    let global_flat = effects.clone().filter(|e| e.modifier == Modifier::Flat);
    apply_effects_to_skill_specs(skill_specs, global_flat);
    apply_effects_to_skill_specs(skill_specs, local_effects.iter());
    apply_effects_to_skill_specs(
        skill_specs,
        effects.filter(|e| e.modifier == Modifier::Multiplier),
    );
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

pub fn compute_skill_upgrade_effects(skill_specs: &SkillSpecs, level: u16) -> EffectsMap {
    // ) -> impl Iterator<Item = StatEffect> + use<'_> + Clone {
    let level = level as f64 - 1.0;
    // skill_specs
    //     .base
    //     .upgrade_effects
    //     .iter()
    //     .map(move |effect| StatEffect {
    //         stat: effect.stat.clone(),
    //         modifier: effect.modifier,
    // value: match effect.modifier {
    //     Modifier::Multiplier if effect.stat.is_multiplicative() => {
    //         ((1.0 + effect.value * 0.01).powf(level) - 1.0) * 100.0
    //     }
    //     _ => effect.value * level,
    // },
    //         bypass_ignore: false,
    //     })

    skill_specs.base.upgrade_effects.iter().fold(
        EffectsMap(HashMap::new()),
        |mut effects_map, effect| {
            *effects_map
                .0
                .entry((effect.stat.clone(), effect.modifier))
                .or_default() += match effect.modifier {
                Modifier::Multiplier if effect.stat.is_multiplicative() => {
                    ((1.0 + effect.value * 0.01).powf(level) - 1.0) * 100.0
                }
                _ => effect.value * level,
            };
            effects_map
        },
    )
}

fn compute_skill_modifier_effects<'a>(
    skill_specs: &'a SkillSpecs,
    inventory: Option<&'a PlayerInventory>,
) -> EffectsMap {
    let item_sources = skill_specs
        .base
        .modifier_effects
        .iter()
        .filter_map(|me| match &me.source {
            ModifierEffectSource::ItemStats { slot, item_stats } => Some((me, *slot, item_stats)),
            _ => None,
        })
        .flat_map(move |(me, slot, item_stats)| {
            inventory
                .into_iter()
                .flat_map(|inv| inv.equipped_items())
                .filter_map(move |(item_slot, item_specs)| {
                    let base = if slot.unwrap_or(item_slot) == item_slot {
                        match (
                            item_stats,
                            &item_specs.weapon_specs,
                            &item_specs.armor_specs,
                        ) {
                            (ItemStatsSource::Damage(damage_type), Some(weapon_specs), _) => {
                                if let Some(dmg_type) = damage_type {
                                    weapon_specs
                                        .damage
                                        .get(dmg_type)
                                        .map(|d| (d.min + d.max) * 0.5)
                                        .unwrap_or_default()
                                } else {
                                    weapon_specs
                                        .damage
                                        .values()
                                        .map(|d| (d.min + d.max) * 0.5)
                                        .sum()
                                }
                            }
                            (ItemStatsSource::Armor, _, Some(armor_specs)) => armor_specs.armor,
                            _ => 0.0,
                        }
                    } else {
                        0.0
                    };

                    if base > 0.0 {
                        Some((me, me.factor * base))
                    } else {
                        None
                    }
                })
        });

    let non_item_sources =
        skill_specs
            .base
            .modifier_effects
            .iter()
            .filter_map(|me| match &me.source {
                ModifierEffectSource::ItemStats { .. } => None,
                ModifierEffectSource::PlaceHolder => todo!(),
            });

    item_sources
        .chain(non_item_sources)
        .flat_map(|(modifier_effect, factor)| {
            modifier_effect.effects.iter().map(move |effect| {
                (
                    (effect.stat.clone(), effect.modifier),
                    effect.value * factor,
                )
            })
        })
        .fold(EffectsMap(HashMap::new()), |mut map, (key, val)| {
            *map.0.entry(key).or_default() += val;
            map
        })
}

// fn compute_skill_modifier_effects<'a>(
//     skill_specs: &'a SkillSpecs,
//     inventory: Option<&'a PlayerInventory>,
// ) -> EffectsMap {
//     // ) -> impl Iterator<Item = StatEffect> + use<'a> + Clone {
//     skill_specs
//         .base
//         .modifier_effects
//         .iter()
//         .flat_map(
//             move |modifier_effect| match (modifier_effect.source, inventory) {
//                 (ModifierEffectSource::ItemStats { slot, item_stats }, Some(inventory)) => {
//                     inventory
//                         .equipped_items()
//                         .filter_map(move |(item_slot, item_specs)| {
//                             let factor = if slot.unwrap_or(item_slot) == item_slot {
//                                 match (
//                                     item_stats,
//                                     &item_specs.weapon_specs,
//                                     &item_specs.armor_specs,
//                                 ) {
//                                     (
//                                         ItemStatsSource::Damage(damage_type),
//                                         Some(weapon_specs),
//                                         _,
//                                     ) => {
//                                         if let Some(damage_type) = damage_type {
//                                             weapon_specs
//                                                 .damage
//                                                 .get(&damage_type)
//                                                 .map(|value| (value.min + value.max) * 0.5)
//                                                 .unwrap_or_default()
//                                         } else {
//                                             weapon_specs
//                                                 .damage
//                                                 .values()
//                                                 .map(|value| (value.min + value.max) * 0.5)
//                                                 .sum()
//                                         }
//                                     }
//                                     (ItemStatsSource::Armor, _, Some(armor_specs)) => {
//                                         armor_specs.armor
//                                     }
//                                     _ => 0.0,
//                                 }
//                             } else {
//                                 0.0
//                             };

//                             if factor > 0.0 {
//                                 Some((modifier_effect, modifier_effect.factor * factor))
//                             } else {
//                                 None
//                             }
//                         })
//                 }
//                 (ModifierEffectSource::ItemStats { .. }, None) => std::iter::empty(),
//             },
//         )
//         .flat_map(|(modifier_effect, factor)| {
//             modifier_effect.effects.iter().map(move |effect| {
//                 (
//                     (effect.stat.clone(), effect.modifier),
//                     effect.value * factor,
//                 )
//             })
//         })
//         .fold(
//             EffectsMap(HashMap::new()),
//             |mut effects_map, (key, value)| {
//                 *effects_map.0.entry(key).or_default() += value;
//                 effects_map
//             },
//         )
// }

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

    // NB: With this approach, Inc Spell Crit Chance is multiplicative with Inc Crit Chance...
    // But maybe that's fine...

    let mut stat_converters = Vec::new();

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

        if effect.stat.is_match(&StatType::SuccessChance {
            skill_type: Some(skill_type),
            effect_type: (&skill_effect.effect_type).into(),
        }) {
            skill_effect
                .failure_chance
                .value
                .apply_negative_effect(effect);
        }

        if let StatType::StatConverter(specs) = &effect.stat {
            stat_converters.push((specs.clone(), effect.value));
            continue;
        }

        match &mut skill_effect.effect_type {
            SkillEffectType::FlatDamage {
                damage,
                crit_chance,
                crit_damage,
                ..
            } => {
                for damage_type in DamageType::iter() {
                    let value = damage.entry(damage_type).or_default();
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
                        status_effect.value.min.apply_effect(effect);
                        status_effect.value.max.apply_effect(effect);
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

    if !stat_converters.is_empty() {
        let mut stats_converted = Vec::with_capacity(stat_converters.len());

        for (specs, factor) in stat_converters {
            if specs.skill_type.is_some_and(|s| s != skill_type) {
                continue;
            }

            if let Some(stat) = match (specs.source, &mut skill_effect.effect_type) {
                (
                    StatConverterSource::CritDamage,
                    SkillEffectType::FlatDamage { crit_damage, .. },
                ) => {
                    let amount = *crit_damage * factor * 0.01;
                    if !specs.is_extra {
                        *crit_damage -= amount;
                    }

                    Some(StatEffect {
                        stat: (*specs.target_stat).clone(),
                        modifier: specs.target_modifier,
                        value: amount,
                        bypass_ignore: true,
                    })
                }
                (
                    StatConverterSource::Damage { damage_type },
                    SkillEffectType::FlatDamage { damage, .. },
                ) => {
                    let amount = match damage_type {
                        Some(damage_type) => damage
                            .get_mut(&damage_type)
                            .map(|d| {
                                let amount = (d.min * factor * 0.01, d.max * factor * 0.01);
                                if !specs.is_extra {
                                    d.min -= amount.0;
                                    d.max -= amount.1;
                                }
                                amount
                            })
                            .unwrap_or_default(),
                        None => damage
                            .values_mut()
                            .fold((0.0, 0.0), |(min_acc, max_acc), d| {
                                let amount = (d.min * factor * 0.01, d.max * factor * 0.01);
                                if !specs.is_extra {
                                    d.min -= amount.0;
                                    d.max -= amount.1;
                                }
                                (min_acc + amount.0, max_acc + amount.1)
                            }),
                    };
                    // Special case, when converting damage we map on min and max respectively
                    if let StatType::Damage {
                        skill_type,
                        damage_type,
                    } = *specs.target_stat
                    {
                        stats_converted.push(StatEffect {
                            stat: StatType::MinDamage {
                                skill_type,
                                damage_type,
                            },
                            modifier: specs.target_modifier,
                            value: amount.0,
                            bypass_ignore: true,
                        });
                        Some(StatEffect {
                            stat: StatType::MaxDamage {
                                skill_type,
                                damage_type,
                            },
                            modifier: specs.target_modifier,
                            value: amount.1,
                            bypass_ignore: true,
                        })
                    } else {
                        Some(StatEffect {
                            stat: (*specs.target_stat).clone(),
                            modifier: specs.target_modifier,
                            value: (amount.0 + amount.1),
                            bypass_ignore: true,
                        })
                    }
                }
                _ => None,
            } {
                stats_converted.push(stat);
            }
        }

        compute_skill_specs_effect(skill_type, skill_effect, stats_converted.iter());
    }

    if let SkillEffectType::FlatDamage { damage, .. } = &mut skill_effect.effect_type {
        damage.retain(|_, value| {
            value.min = value.min.max(0.0);
            value.max = value.max.max(0.0);
            value.clamp();

            value.max > 0.0
        });
    }
}
