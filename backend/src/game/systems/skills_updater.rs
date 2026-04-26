use std::{collections::HashMap, time::Duration};
use strum::IntoEnumIterator;

use shared::data::{
    chance::ChanceRange,
    character::CharacterAttrs,
    character_status::StatusSpecs,
    conditional_modifier::ConditionalModifier,
    modifier::Modifier,
    player::PlayerInventory,
    skill::{
        BaseSkillSpecs, DamageType, ItemStatsSource, ModifierEffectSource, RepeatedSkillEffect,
        SkillEffect, SkillEffectType, SkillSpecs, SkillState, SkillType,
    },
    stat_effect::{
        EffectsMap, LuckyRollType, Matchable, MinMax, StatConverterSource, StatEffect, StatType,
        compare_options,
    },
};

use crate::game::systems::characters_updater;

pub fn update_skills_states(
    elapsed_time: Duration,
    skills_specs: &[SkillSpecs],
    skills_states: &mut [SkillState],
) {
    for (skill_specs, skill_state) in skills_specs.iter().zip(skills_states.iter_mut()) {
        if skill_specs.cooldown.get() > 0.0 {
            skill_state.elapsed_cooldown +=
                (elapsed_time.as_secs_f64() / skill_specs.cooldown.get()).into();
        }
        skill_state.is_ready = skill_state.elapsed_cooldown.get() >= 1.0;
    }
}

pub fn update_repeated_skill_effects(
    elapsed_time: Duration,
    repeated_skill_effectd: &mut [RepeatedSkillEffect],
) {
    for repeated_skill_effect in repeated_skill_effectd.iter_mut() {
        if repeated_skill_effect
            .targets_group
            .repeat
            .repeat_cooldown
            .get()
            > 0.0
        {
            repeated_skill_effect.elapsed_cooldown += (elapsed_time.as_secs_f64()
                / repeated_skill_effect
                    .targets_group
                    .repeat
                    .repeat_cooldown
                    .get())
            .into();
        }
    }
}

pub fn reset_skills(skills_states: &mut [SkillState]) {
    for skill_state in skills_states.iter_mut() {
        skill_state.just_triggered = false;
    }
}

pub fn update_skill_specs(
    skill_id: String,
    base_skill_specs: &BaseSkillSpecs,
    upgrade_level: u16,
    effects: &[StatEffect],
    character_attrs: &CharacterAttrs,
    inventory: Option<&PlayerInventory>,
) -> SkillSpecs {
    let level_modifier = effects
        .iter()
        .map(|e| {
            if let StatType::SkillLevel(skill_filter) = &e.stat
                && skill_filter.is_match_with_skill(base_skill_specs.skill_type, &skill_id)
                && e.modifier == Modifier::Flat
            // if e.modifier == Modifier::Flat
            //     && e.stat
            //         .is_match(&StatType::SkillLevel(Some(skill_specs.base.skill_type)))
            {
                e.value as u16
            } else {
                0
            }
        })
        .sum();

    let mut skill_specs = SkillSpecs {
        skill_id,
        name: base_skill_specs.name.clone(),
        icon: base_skill_specs.icon.clone(),
        description: base_skill_specs.description.clone(),
        skill_type: base_skill_specs.skill_type,
        ignore_stat_effects: base_skill_specs.ignore_stat_effects.clone(),
        cooldown: base_skill_specs.cooldown.into(),
        mana_cost: base_skill_specs.mana_cost.into(),
        targets: base_skill_specs.targets.clone(),
        triggers: base_skill_specs.triggers.clone(),
        level_modifier,
    };

    let local_effects: Vec<_> = (&EffectsMap::combine_all(
        std::iter::once(compute_skill_upgrade_effects(
            base_skill_specs,
            upgrade_level.saturating_add(skill_specs.level_modifier),
        ))
        .chain(std::iter::once(compute_skill_modifier_effects(
            base_skill_specs,
            character_attrs,
            inventory,
        ))),
    ))
        .into();

    apply_effects_to_skill_specs(&mut skill_specs, local_effects.iter().chain(effects));

    skill_specs
}

pub fn apply_effects_to_skill_specs<'a>(
    skill_specs: &mut SkillSpecs,
    effects: impl Iterator<Item = &'a StatEffect> + Clone,
) {
    let effects = effects.filter(|effect| {
        effect.bypass_ignore
            || skill_specs
                .ignore_stat_effects
                .iter()
                .all(|ignore| !effect.stat.is_match(ignore))
    });

    for effect in effects.clone() {
        if let StatType::Speed(skill_filter) = &effect.stat
            && skill_filter.is_match_with_skill(skill_specs.skill_type, &skill_specs.skill_id)
        {
            skill_specs.cooldown.apply_negative_effect(effect);
        }

        if let StatType::ManaCost { skill_filter } = &effect.stat
            && skill_filter.is_match_with_skill(skill_specs.skill_type, &skill_specs.skill_id)
        {
            skill_specs.mana_cost.apply_effect(effect);
        }

        if let StatType::SkillTargetModifier {
            skill_filter,
            range,
            shape,
            repeat,
        } = &effect.stat
            && skill_filter.is_match_with_skill(skill_specs.skill_type, &skill_specs.skill_id)
        {
            for target in skill_specs.targets.iter_mut() {
                if let Some(range) = range {
                    target.range = *range;
                }
                if let Some(shape) = shape {
                    target.shape = *shape;
                }
                if let Some(repeat) = repeat {
                    target.repeat.target = repeat.target;
                    target.repeat.value = ChanceRange {
                        min: repeat.min_value,
                        max: repeat.max_value,
                        lucky_chance: Default::default(),
                    };
                }
            }

            // TODO: Triggers
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
        compute_skill_specs_effect(
            &skill_specs.skill_id,
            skill_specs.skill_type,
            skill_effect,
            effects.clone(),
        )
    }
}

pub fn compute_skill_upgrade_effects(base_skill_specs: &BaseSkillSpecs, level: u16) -> EffectsMap {
    let level = level as f64 - 1.0;

    base_skill_specs.upgrade_effects.iter().fold(
        EffectsMap(HashMap::new()),
        |mut effects_map, effect| {
            *effects_map
                .0
                .entry((effect.stat.clone(), effect.modifier, effect.bypass_ignore))
                .or_default() += match effect.modifier {
                Modifier::More => ((1.0 + effect.value * 0.01).powf(level) - 1.0) * 100.0,
                _ => effect.value * level,
            };
            effects_map
        },
    )
}

fn compute_skill_modifier_effects<'a>(
    base_skill_specs: &'a BaseSkillSpecs,
    character_attrs: &CharacterAttrs,
    inventory: Option<&'a PlayerInventory>,
) -> EffectsMap {
    let item_sources: Vec<_> = base_skill_specs
        .modifier_effects
        .iter()
        .filter_map(|modifier_effect| match &modifier_effect.source {
            ModifierEffectSource::ItemStats {
                slot,
                category,
                item_stats,
            } => Some((modifier_effect, *slot, *category, item_stats)),
            _ => None,
        })
        .flat_map(move |(modifier_effect, slot, category, item_stats)| {
            inventory
                .into_iter()
                .flat_map(|inv| inv.equipped_items())
                .filter_map(move |(item_slot, item_specs)| {
                    let mut modifier_effect = modifier_effect.clone();
                    let slot = slot.unwrap_or(item_slot);

                    let base = if (slot == item_slot || item_specs.base.extra_slots.contains(&slot))
                        && category
                            .map(|category| item_specs.base.categories.contains(&category))
                            .unwrap_or(true)
                    {
                        match (
                            item_stats,
                            &item_specs.weapon_specs,
                            &item_specs.armor_specs,
                        ) {
                            (ItemStatsSource::Armor, _, Some(armor_specs)) => *armor_specs.armor,
                            (ItemStatsSource::Block, _, Some(armor_specs)) => {
                                armor_specs.block.get() as f64
                            }
                            (ItemStatsSource::Cooldown, Some(weapon_specs), _) => {
                                weapon_specs.cooldown.get()
                            }
                            (ItemStatsSource::CritChance, Some(weapon_specs), _) => {
                                weapon_specs.crit_chance.value.get() as f64
                            }
                            (ItemStatsSource::CritDamage, Some(weapon_specs), _) => {
                                *weapon_specs.crit_damage
                            }
                            (
                                ItemStatsSource::Damage {
                                    damage_type,
                                    min_max,
                                },
                                Some(weapon_specs),
                                _,
                            ) => {
                                if let Some(dmg_type) = damage_type {
                                    weapon_specs
                                        .damage
                                        .get(dmg_type)
                                        .map(|d| match min_max {
                                            Some(MinMax::Min) => d.min.get(),
                                            Some(MinMax::Max) => d.max.get(),
                                            None => (d.min.get() + d.max.get()) * 0.5,
                                        })
                                        .unwrap_or_default()
                                } else {
                                    weapon_specs
                                        .damage
                                        .values()
                                        .map(|d| match min_max {
                                            Some(MinMax::Min) => d.min.get(),
                                            Some(MinMax::Max) => d.max.get(),
                                            None => (d.min.get() + d.max.get()) * 0.5,
                                        })
                                        .sum()
                                }
                            }
                            (ItemStatsSource::Range, Some(weapon_specs), _) => {
                                for effect in modifier_effect.effects.iter_mut() {
                                    if let StatType::SkillTargetModifier { range, .. } =
                                        &mut effect.stat
                                    {
                                        *range = Some(weapon_specs.range);
                                    }
                                }
                                1.0
                            }
                            (ItemStatsSource::Shape, Some(weapon_specs), _) => {
                                for effect in modifier_effect.effects.iter_mut() {
                                    if let StatType::SkillTargetModifier { shape, .. } =
                                        &mut effect.stat
                                    {
                                        *shape = Some(weapon_specs.shape);
                                    }
                                }
                                1.0
                            }
                            _ => 0.0,
                        }
                    } else {
                        0.0
                    };

                    if base > 0.0 {
                        let factor = modifier_effect.factor * base;
                        Some((modifier_effect, factor))
                    } else {
                        None
                    }
                })
        })
        .collect();

    let non_item_sources: Vec<_> = base_skill_specs
        .modifier_effects
        .iter()
        .filter_map(|me| match &me.source {
            ModifierEffectSource::ItemStats { .. } => None,
            ModifierEffectSource::CharacterStats(stat_converter) => Some((
                me.clone(),
                me.factor
                    * characters_updater::compute_stat_converter(character_attrs, stat_converter),
            )),
        })
        .collect();

    item_sources
        .iter()
        .chain(non_item_sources.iter())
        .flat_map(|(modifier_effect, factor)| {
            modifier_effect.effects.iter().map(move |effect| {
                (
                    (effect.stat.clone(), effect.modifier, effect.bypass_ignore),
                    effect.value * factor,
                )
            })
        })
        .fold(EffectsMap(HashMap::new()), |mut map, (key, val)| {
            *map.0.entry(key).or_default() += val;
            map
        })
}

pub fn compute_skill_specs_effect<'a>(
    skill_id: &String,
    skill_type: SkillType,
    skill_effect: &mut SkillEffect,
    effects: impl Iterator<Item = &'a StatEffect> + Clone,
) {
    if let SkillEffectType::ApplyStatus { statuses, .. } = &mut skill_effect.effect_type {
        for status_effect in statuses.iter_mut() {
            if let StatusSpecs::Trigger(ref mut trigger_specs) = status_effect.status_type {
                for triggered_effect in trigger_specs.triggered_effect.effects.iter_mut() {
                    compute_skill_specs_effect(
                        skill_id,
                        skill_type,
                        triggered_effect,
                        effects.clone(),
                    )
                }
            }
        }
    }

    let mut stats_converters = Vec::new();

    for effect in effects.clone() {
        if !effect.bypass_ignore
            && skill_effect
                .ignore_stat_effects
                .iter()
                .any(|ignore| effect.stat.is_match(ignore))
        {
            continue;
        }

        if let StatType::Lucky {
            skill_filter,
            roll_type: LuckyRollType::SuccessChance { effect_type },
        } = &effect.stat
            && skill_filter.is_match_with_skill(skill_type, skill_id)
            && compare_options(effect_type, &(&skill_effect.effect_type).into())
        {
            skill_effect
                .success_chance
                .lucky_chance
                .apply_effect(effect);
            continue;
        }

        if let StatType::SuccessChance {
            skill_filter,
            effect_type,
        } = &effect.stat
            && skill_filter.is_match_with_skill(skill_type, skill_id)
            && compare_options(effect_type, &(&skill_effect.effect_type).into())
        {
            skill_effect.success_chance.value.apply_effect(effect);
            continue;
        }

        if let StatType::StatConverter(specs) = &effect.stat {
            stats_converters.push((specs.clone(), effect.modifier, effect.value));
            continue;
        }

        if let StatType::SkillConditionalModifier {
            skill_filter,
            conditions,
            stat,
        } = &effect.stat
            && skill_filter.is_match_with_skill(skill_type, skill_id)
        {
            skill_effect
                .conditional_modifiers
                .push(ConditionalModifier {
                    conditions: conditions.clone(),
                    conditions_duration: 0,
                    effects: [StatEffect {
                        stat: *(stat.clone()),
                        modifier: effect.modifier,
                        value: effect.value,
                        bypass_ignore: effect.bypass_ignore,
                    }]
                    .into(),
                });
            continue;
        }

        match &mut skill_effect.effect_type {
            SkillEffectType::FlatDamage {
                damage,
                crit_chance,
                crit_damage,
                ..
            } => {
                for damage_type in DamageType::iter().filter(|d| *d != DamageType::Poison) {
                    let value = damage.entry(damage_type).or_default();

                    if let StatType::Damage {
                        skill_filter,
                        damage_type: stat_damage_type,
                        min_max,
                        is_hit,
                    } = &effect.stat
                        && skill_filter.is_match_with_skill(skill_type, skill_id)
                        && compare_options(stat_damage_type, &Some(damage_type))
                        && compare_options(is_hit, &Some(true))
                    {
                        if compare_options(min_max, &Some(MinMax::Min)) {
                            value.min.apply_effect(effect);
                        }

                        if compare_options(min_max, &Some(MinMax::Max)) {
                            value.max.apply_effect(effect);
                        }
                    }

                    if let StatType::Lucky {
                        skill_filter,
                        roll_type:
                            LuckyRollType::Damage {
                                damage_type: stat_damage_type,
                            },
                    } = &effect.stat
                        && skill_filter.is_match_with_skill(skill_type, skill_id)
                        && compare_options(stat_damage_type, &Some(damage_type))
                    {
                        value.lucky_chance.apply_effect(effect);
                    }
                }

                if let StatType::CritChance(skill_filter) = &effect.stat
                    && skill_filter.is_match_with_skill(skill_type, skill_id)
                {
                    crit_chance.value.apply_effect(effect);
                }

                if let StatType::Lucky {
                    skill_filter,
                    roll_type: LuckyRollType::CritChance,
                } = &effect.stat
                    && skill_filter.is_match_with_skill(skill_type, skill_id)
                {
                    crit_chance.lucky_chance.apply_effect(effect);
                }

                if let StatType::CritDamage(skill_filter) = &effect.stat
                    && skill_filter.is_match_with_skill(skill_type, skill_id)
                {
                    crit_damage.apply_effect(effect);
                }

                // crit_chance.clamp();
            }
            SkillEffectType::ApplyStatus { statuses, duration } => {
                if let StatType::StatusDuration {
                    status_type,
                    skill_filter,
                } = &effect.stat
                    && statuses.iter().any(|status_effect| {
                        compare_options(status_type, &Some((&status_effect.status_type).into()))
                    })
                    && skill_filter.is_match_with_skill(skill_type, skill_id)
                {
                    duration.min.apply_effect(effect);
                    duration.max.apply_effect(effect);
                }

                for status_effect in statuses.iter_mut() {
                    if let StatType::StatusPower {
                        status_type,
                        skill_filter,
                        min_max,
                    } = &effect.stat
                        && compare_options(status_type, &Some((&status_effect.status_type).into()))
                        && skill_filter.is_match_with_skill(skill_type, skill_id)
                    {
                        if compare_options(min_max, &Some(MinMax::Min)) {
                            status_effect.value.min.apply_effect(effect);
                        }
                        if compare_options(min_max, &Some(MinMax::Max)) {
                            status_effect.value.max.apply_effect(effect);
                        }
                    }

                    if let StatusSpecs::DamageOverTime { damage_type, .. } =
                        status_effect.status_type
                    {
                        if let StatType::Damage {
                            skill_filter,
                            damage_type: stat_damage_type,
                            min_max,
                            is_hit,
                        } = &effect.stat
                            && skill_filter.is_match_with_skill(skill_type, skill_id)
                            && compare_options(stat_damage_type, &Some(damage_type))
                            && compare_options(is_hit, &Some(false))
                        {
                            if compare_options(min_max, &Some(MinMax::Min)) {
                                status_effect.value.min.apply_effect(effect);
                            }
                            if compare_options(min_max, &Some(MinMax::Max)) {
                                status_effect.value.max.apply_effect(effect);
                            }
                        }

                        if let StatType::Lucky {
                            skill_filter,
                            roll_type:
                                LuckyRollType::Damage {
                                    damage_type: stat_damage_type,
                                },
                        } = &effect.stat
                            && skill_filter.is_match_with_skill(skill_type, skill_id)
                            && compare_options(stat_damage_type, &Some(damage_type))
                        {
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
                if let StatType::Restore {
                    restore_type: stat_restore_type,
                    skill_filter,
                } = &effect.stat
                    && compare_options(stat_restore_type, &Some(*restore_type))
                    && skill_filter.is_match_with_skill(skill_type, skill_id)
                {
                    value.min.apply_effect(effect);
                    value.max.apply_effect(effect);
                };
            }
            SkillEffectType::Resurrect => {}
            SkillEffectType::RefreshCooldown {
                skill_filter: _,
                value: _,
                modifier: _,
            } => {}
        }
    }

    if !stats_converters.is_empty() {
        stats_converters.sort_by_key(|(stat_converter, modifier, _)| {
            (
                stat_converter.source,
                stat_converter.stat.clone(),
                *modifier,
            )
        });

        let mut stats_converted = Vec::with_capacity(stats_converters.len());

        for (specs, modifier, factor) in stats_converters {
            if specs.skill_type.is_some_and(|s| s != skill_type) {
                continue;
            }

            if let Some(stat) = match (specs.source, &mut skill_effect.effect_type) {
                (
                    StatConverterSource::CritDamage,
                    SkillEffectType::FlatDamage { crit_damage, .. },
                ) => {
                    let amount = crit_damage.convert_value(factor, specs.is_extra, false);

                    (amount > 0.0).then(|| StatEffect {
                        stat: (*specs.stat).clone(),
                        modifier,
                        value: amount,
                        bypass_ignore: true,
                    })
                }
                (
                    StatConverterSource::Damage {
                        damage_type,
                        min_max,
                    },
                    SkillEffectType::FlatDamage { damage, .. },
                ) => {
                    let min_factor = if let Some(MinMax::Min) | None = min_max {
                        factor
                    } else {
                        0.0
                    };
                    let max_factor = if let Some(MinMax::Max) | None = min_max {
                        factor
                    } else {
                        0.0
                    };
                    let amount = match damage_type {
                        Some(damage_type) => damage
                            .get_mut(&damage_type)
                            .map(|d| {
                                (
                                    d.min.convert_value(min_factor, specs.is_extra, true).get(),
                                    d.max.convert_value(max_factor, specs.is_extra, true).get(),
                                )
                            })
                            .unwrap_or_default(),
                        None => damage
                            .values_mut()
                            .fold((0.0, 0.0), |(min_acc, max_acc), d| {
                                (
                                    min_acc
                                        + d.min
                                            .convert_value(min_factor, specs.is_extra, true)
                                            .get(),
                                    max_acc
                                        + d.max
                                            .convert_value(max_factor, specs.is_extra, true)
                                            .get(),
                                )
                            }),
                    };

                    // Special case, when converting damage we map on min and max respectively
                    if let None = min_max
                        && let StatType::Damage {
                            skill_filter,
                            damage_type,
                            min_max: None,
                            is_hit,
                        } = *specs.stat
                    {
                        stats_converted.push(StatEffect {
                            stat: StatType::Damage {
                                skill_filter: skill_filter.clone(),
                                damage_type,
                                min_max: Some(MinMax::Min),
                                is_hit,
                            },
                            modifier,
                            value: amount.0,
                            bypass_ignore: true,
                        });
                        Some(StatEffect {
                            stat: StatType::Damage {
                                skill_filter,
                                damage_type,
                                min_max: Some(MinMax::Max),
                                is_hit,
                            },
                            modifier,
                            value: amount.1,
                            bypass_ignore: true,
                        })
                    } else {
                        Some(StatEffect {
                            stat: (*specs.stat).clone(),
                            modifier,
                            value: (amount.0 + amount.1),
                            bypass_ignore: true,
                        })
                    }
                }
                // (
                //     StatConverterSource::DamageOverTime {
                //         damage_type,
                //         min_max,
                //     },
                //     SkillEffectType::ApplyStatus { statuses, .. },
                // ) => {
                //     let min_factor = if let Some(MinMax::Min) | None = min_max {
                //         factor
                //     } else {
                //         0.0
                //     };
                //     let max_factor = if let Some(MinMax::Max) | None = min_max {
                //         factor
                //     } else {
                //         0.0
                //     };
                //     let amount: (f64, f64) = statuses
                //         .iter_mut()
                //         .flat_map(|status| match status.status_type {
                //             StatusSpecs::DamageOverTime {
                //                 damage_type: status_damage_type,
                //             } if status_damage_type
                //                 == damage_type.unwrap_or(status_damage_type) =>
                //             {
                //                 Some((
                //                     status
                //                         .value
                //                         .min
                //                         .convert_value(min_factor, specs.is_extra, true)
                //                         .get(),
                //                     status
                //                         .value
                //                         .max
                //                         .convert_value(max_factor, specs.is_extra, true)
                //                         .get(),
                //                 ))
                //             }
                //             _ => None,
                //         })
                //         .fold((0.0, 0.0), |(a, b), (c, d)| (a + c, b + d));

                //     Some(StatEffect {
                //         stat: (*specs.stat).clone(),
                //         modifier,
                //         value: (amount.0 + amount.1),
                //         bypass_ignore: true,
                //     })
                // }
                _ => None,
            } {
                stats_converted.push(stat);
            }
        }

        compute_skill_specs_effect(skill_id, skill_type, skill_effect, stats_converted.iter());
    }
}
