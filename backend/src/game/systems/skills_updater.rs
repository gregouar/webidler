use std::{
    collections::{HashMap, HashSet},
    time::Duration,
};

use shared::data::{
    character_status::StatusSpecs,
    conditional_modifier::ConditionalModifier,
    player::PlayerInventory,
    skill::{
        ApplyStatusEffect, DamageType, ItemStatsSource, ModifierEffectSource, RestoreType,
        SkillEffect, SkillEffectType, SkillSpecs, SkillState, SkillType,
    },
    stat_effect::{
        EffectsMap, LuckyRollType, MinMax, Modifier, StatConverterSource, StatEffect,
        StatSkillEffectType, StatType, compare_options,
    },
};
use strum::IntoEnumIterator;

use crate::game::utils::{
    modifiable_value::{
        ModifiableChance, ModifiableChanceRange, ModifiableDamageMap, ModifiableValue,
        to_modifiable_damage_map,
    },
    rng::Rollable,
};

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

pub fn update_skill_specs(
    skill_specs: &mut SkillSpecs,
    effects: &[StatEffect],
    inventory: Option<&PlayerInventory>,
) {
    skill_specs.targets = skill_specs.base.targets.clone();
    skill_specs.triggers = skill_specs.base.triggers.clone();
    skill_specs.cooldown = skill_specs.base.cooldown.into();
    skill_specs.mana_cost = skill_specs.base.mana_cost.into();

    skill_specs.level_modifier = effects
        .iter()
        .map(|e| {
            if e.modifier == Modifier::Flat
                && e.stat
                    .is_match(&StatType::SkillLevel(Some(skill_specs.base.skill_type)))
            {
                e.value as u16
            } else {
                0
            }
        })
        .sum();

    let local_effects: Vec<_> = (&EffectsMap::combine_all(
        std::iter::once(compute_skill_upgrade_effects(
            skill_specs,
            skill_specs
                .upgrade_level
                .saturating_add(skill_specs.level_modifier),
        ))
        .chain(std::iter::once(compute_skill_modifier_effects(
            skill_specs,
            inventory,
        ))),
    ))
        .into();

    apply_effects_to_skill_specs(skill_specs, local_effects.iter().chain(effects));
}

pub fn apply_effects_to_skill_specs<'a>(
    skill_specs: &mut SkillSpecs,
    effects: impl Iterator<Item = &'a StatEffect> + Clone,
) {
    let mut modifiable_skill_specs = ModifiableSkillSpecs {
        cooldown: skill_specs.cooldown.into(),
        mana_cost: skill_specs.mana_cost.into(),
    };

    for effect in effects.clone() {
        if effect
            .stat
            .is_match(&StatType::Speed(Some(skill_specs.base.skill_type)))
        {
            modifiable_skill_specs
                .cooldown
                .apply_negative_effect(effect);
        }

        if effect.stat.is_match(&StatType::ManaCost {
            skill_type: Some(skill_specs.base.skill_type),
        }) {
            modifiable_skill_specs.mana_cost.apply_effect(effect);
        }

        if let StatType::SkillTargetModifier {
            skill_type,
            range,
            shape,
        } = &effect.stat
            && compare_options(skill_type, &Some(skill_specs.base.skill_type))
        {
            for target in skill_specs.targets.iter_mut() {
                if let Some(range) = range {
                    target.range = *range;
                }
                if let Some(shape) = shape {
                    target.shape = *shape;
                }
            }

            // TODO: Triggers
        }
    }

    let ModifiableSkillSpecs {
        cooldown,
        mana_cost,
    } = modifiable_skill_specs;

    skill_specs.cooldown = cooldown.evaluate();
    skill_specs.mana_cost = mana_cost.evaluate();

    for skill_effect in skill_specs
        .targets
        .iter_mut()
        .flat_map(|t| t.effects.iter_mut())
        .chain(
            skill_specs
                .triggers
                .iter_mut()
                .filter(|trigger| trigger.triggered_effect.inherit_modifiers)
                .flat_map(|trigger| trigger.triggered_effect.effects.iter_mut()),
        )
    {
        compute_skill_specs_effect(skill_specs.base.skill_type, skill_effect, effects.clone())
    }
}

pub fn compute_skill_upgrade_effects(skill_specs: &SkillSpecs, level: u16) -> EffectsMap {
    let level = level as f64 - 1.0;

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
    let item_sources: Vec<_> = skill_specs
        .base
        .modifier_effects
        .iter()
        .filter_map(|modifier_effect| match &modifier_effect.source {
            ModifierEffectSource::ItemStats { slot, item_stats } => {
                Some((modifier_effect, *slot, item_stats))
            }
            _ => None,
        })
        .flat_map(move |(modifier_effect, slot, item_stats)| {
            inventory
                .into_iter()
                .flat_map(|inv| inv.equipped_items())
                .filter_map(move |(item_slot, item_specs)| {
                    let mut modifier_effect = modifier_effect.clone();
                    let base = if slot.unwrap_or(item_slot) == item_slot
                        || item_specs.base.extra_slots.contains(&item_slot)
                    {
                        match (
                            item_stats,
                            &item_specs.weapon_specs,
                            &item_specs.armor_specs,
                        ) {
                            (ItemStatsSource::Armor, _, Some(armor_specs)) => armor_specs.armor,
                            (ItemStatsSource::Cooldown, Some(weapon_specs), _) => {
                                weapon_specs.cooldown as f64
                            }
                            (ItemStatsSource::CritChance, Some(weapon_specs), _) => {
                                weapon_specs.crit_chance.value as f64
                            }
                            (ItemStatsSource::CritDamage, Some(weapon_specs), _) => {
                                weapon_specs.crit_damage
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
                                            Some(MinMax::Min) => d.min,
                                            Some(MinMax::Max) => d.max,
                                            None => (d.min + d.max) * 0.5,
                                        })
                                        .unwrap_or_default()
                                } else {
                                    weapon_specs
                                        .damage
                                        .values()
                                        .map(|d| match min_max {
                                            Some(MinMax::Min) => d.min,
                                            Some(MinMax::Max) => d.max,
                                            None => (d.min + d.max) * 0.5,
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

    let non_item_sources: Vec<_> = skill_specs
        .base
        .modifier_effects
        .iter()
        .filter_map(|me| match &me.source {
            ModifierEffectSource::ItemStats { .. } => None,
            ModifierEffectSource::PlaceHolder => todo!(),
        })
        .collect();

    item_sources
        .iter()
        .chain(non_item_sources.iter())
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

pub fn compute_skill_specs_effect<'a>(
    skill_type: SkillType,
    skill_effect: &mut SkillEffect,
    effects: impl Iterator<Item = &'a StatEffect> + Clone,
) {
    if let SkillEffectType::ApplyStatus { statuses, .. } = &mut skill_effect.effect_type {
        for status_effect in statuses.iter_mut() {
            if let StatusSpecs::Trigger(ref mut trigger_specs) = status_effect.status_type
                && trigger_specs.triggered_effect.inherit_modifiers
            {
                for triggered_effect in trigger_specs.triggered_effect.effects.iter_mut() {
                    compute_skill_specs_effect(skill_type, triggered_effect, effects.clone())
                }
            }
        }
    }

    (*skill_effect) =
        modify_skill_specs_effect(skill_type, skill_effect.into(), effects).evaluate();

    if let SkillEffectType::FlatDamage {
        damage,
        crit_chance,
        ..
    } = &mut skill_effect.effect_type
    {
        crit_chance.clamp();
        damage.retain(|_, value| {
            value.min = value.min.max(0.0).into();
            value.max = value.max.max(0.0).into();
            value.clamp();

            value.max > 0.0
        });
    }
}

fn modify_skill_specs_effect<'a>(
    skill_type: SkillType,
    mut skill_effect: ModifiableSkillEffect,
    effects: impl Iterator<Item = &'a StatEffect> + Clone,
) -> ModifiableSkillEffect {
    use ModifiableSkillEffectType::*;

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
            roll_type: LuckyRollType::SuccessChance {
                effect_type: Some((&skill_effect.effect_type).into()),
            },
        }) {
            skill_effect
                .success_chance
                .lucky_chance
                .apply_effect(effect);
        }

        if effect.stat.is_match(&StatType::SuccessChance {
            skill_type: Some(skill_type),
            effect_type: Some((&skill_effect.effect_type).into()),
        }) {
            skill_effect.success_chance.value.apply_effect(effect);
        }

        if let StatType::StatConverter(specs) = &effect.stat {
            stat_converters.push((specs.clone(), effect.value, effect.modifier));
            continue;
        }

        if let StatType::SkillConditionalModifier {
            skill_type: modifier_skill_type,
            conditions,
            stat,
        } = &effect.stat
            && compare_options(modifier_skill_type, &Some(skill_type))
        {
            skill_effect
                .conditional_modifiers
                .push(ConditionalModifier {
                    conditions: conditions.clone(),
                    effects: [StatEffect {
                        stat: *(stat.clone()),
                        modifier: effect.modifier,
                        value: effect.value,
                        bypass_ignore: effect.bypass_ignore,
                    }]
                    .into(),
                });
        }

        match &mut skill_effect.effect_type {
            FlatDamage {
                damage,
                crit_chance,
                crit_damage,
                ..
            } => {
                for damage_type in DamageType::iter() {
                    let value = damage.entry(damage_type).or_default();

                    if effect.stat.is_match(&StatType::Damage {
                        skill_type: Some(skill_type),
                        damage_type: Some(damage_type),
                        min_max: Some(MinMax::Min),
                    }) {
                        value.min.apply_effect(effect);
                    }

                    if effect.stat.is_match(&StatType::Damage {
                        skill_type: Some(skill_type),
                        damage_type: Some(damage_type),
                        min_max: Some(MinMax::Max),
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

                if effect.stat.is_match(&StatType::Lucky {
                    skill_type: Some(skill_type),
                    roll_type: LuckyRollType::CritChance,
                }) {
                    crit_chance.lucky_chance.apply_effect(effect);
                }

                if effect
                    .stat
                    .is_match(&StatType::CritDamage(Some(skill_type)))
                {
                    crit_damage.apply_effect(effect);
                }
            }
            ApplyStatus { statuses, duration } => {
                if statuses.iter().any(|status_effect| {
                    effect.stat.is_match(&StatType::StatusDuration {
                        status_type: Some((&status_effect.status_type).into()),
                        skill_type: Some(skill_type),
                    })
                }) {
                    duration.min.apply_effect(effect);
                    duration.max.apply_effect(effect);
                }

                for status_effect in statuses.iter_mut() {
                    if effect.stat.is_match(&StatType::StatusPower {
                        status_type: Some((&status_effect.status_type).into()),
                        skill_type: Some(skill_type),
                        min_max: Some(MinMax::Min),
                    }) {
                        status_effect.value.min.apply_effect(effect);
                    }
                    if effect.stat.is_match(&StatType::StatusPower {
                        status_type: Some((&status_effect.status_type).into()),
                        skill_type: Some(skill_type),
                        min_max: Some(MinMax::Max),
                    }) {
                        status_effect.value.max.apply_effect(effect);
                    }

                    if let StatusSpecs::DamageOverTime { damage_type, .. } =
                        status_effect.status_type
                    {
                        if effect.stat.is_match(&StatType::Damage {
                            skill_type: Some(skill_type),
                            damage_type: Some(damage_type),
                            min_max: Some(MinMax::Min),
                        }) {
                            status_effect.value.min.apply_effect(effect);
                        }

                        if effect.stat.is_match(&StatType::Damage {
                            skill_type: Some(skill_type),
                            damage_type: Some(damage_type),
                            min_max: Some(MinMax::Max),
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
            Restore {
                restore_type,
                value,
                ..
            } => {
                if effect.stat.is_match(&StatType::Restore {
                    restore_type: Some(*restore_type),
                    skill_type: Some(skill_type),
                }) {
                    value.min.apply_effect(effect);
                    value.max.apply_effect(effect);
                };
            }
            Resurrect => {}
        }
    }

    if !stat_converters.is_empty() {
        stat_converters.sort_by_key(|(stat_converter, _, modifier)| {
            (
                stat_converter.source,
                stat_converter.stat.clone(),
                *modifier,
            )
        });

        let mut stats_converted = Vec::with_capacity(stat_converters.len());

        for (specs, factor, modifier) in stat_converters {
            if specs.skill_type.is_some_and(|s| s != skill_type) {
                continue;
            }

            if let Some(stat) = match (specs.source, &mut skill_effect.effect_type) {
                (StatConverterSource::CritDamage, FlatDamage { crit_damage, .. }) => {
                    let amount = crit_damage.convert_value(factor, specs.is_extra, false);

                    (amount > 0.0).then(|| StatEffect {
                        stat: (*specs.stat).clone(),
                        modifier,
                        value: amount,
                        bypass_ignore: true,
                    })
                }
                // TODO: Apply status?
                (
                    StatConverterSource::Damage {
                        damage_type,
                        min_max,
                    },
                    FlatDamage { damage, .. },
                ) => {
                    let min_factor = if let Some(MinMax::Min) | None = min_max {
                        factor
                    } else {
                        0.0
                    };
                    let max_factor = if let Some(MinMax::Min) | None = min_max {
                        factor
                    } else {
                        0.0
                    };
                    let amount = match damage_type {
                        Some(damage_type) => damage
                            .get_mut(&damage_type)
                            .map(|d| {
                                (
                                    d.min.convert_value(min_factor, specs.is_extra, true),
                                    d.max.convert_value(max_factor, specs.is_extra, true),
                                )
                            })
                            .unwrap_or_default(),
                        None => damage
                            .values_mut()
                            .fold((0.0, 0.0), |(min_acc, max_acc), d| {
                                (
                                    min_acc + d.min.convert_value(min_factor, specs.is_extra, true),
                                    max_acc + d.max.convert_value(max_factor, specs.is_extra, true),
                                )
                            }),
                    };

                    // Special case, when converting damage we map on min and max respectively
                    if let None = min_max
                        && let StatType::Damage {
                            skill_type,
                            damage_type,
                            min_max: None,
                        } = *specs.stat
                    {
                        stats_converted.push(StatEffect {
                            stat: StatType::Damage {
                                skill_type,
                                damage_type,
                                min_max: Some(MinMax::Min),
                            },
                            modifier,
                            value: amount.0,
                            bypass_ignore: true,
                        });
                        Some(StatEffect {
                            stat: StatType::Damage {
                                skill_type,
                                damage_type,
                                min_max: Some(MinMax::Max),
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
                _ => None,
            } {
                stats_converted.push(stat);
            }
        }

        skill_effect = modify_skill_specs_effect(skill_type, skill_effect, stats_converted.iter());
    }

    skill_effect
}

// Modifiable structs
// ------------------

#[derive(Debug, Clone)]
pub struct ModifiableSkillSpecs {
    cooldown: ModifiableValue<f32>,
    mana_cost: ModifiableValue<f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModifiableSkillEffect {
    success_chance: ModifiableChance,
    effect_type: ModifiableSkillEffectType,
    ignore_stat_effects: HashSet<StatType>,
    conditional_modifiers: Vec<ConditionalModifier>,
}

impl From<&mut SkillEffect> for ModifiableSkillEffect {
    fn from(value: &mut SkillEffect) -> Self {
        Self {
            success_chance: value.success_chance.into(),
            effect_type: (&value.effect_type).into(),
            ignore_stat_effects: value.ignore_stat_effects.clone(), // TODO: don't clone?
            conditional_modifiers: value.conditional_modifiers.clone(),
        }
    }
}
impl ModifiableSkillEffect {
    fn evaluate(self) -> SkillEffect {
        SkillEffect {
            success_chance: self.success_chance.evaluate(),
            effect_type: self.effect_type.evaluate(),
            ignore_stat_effects: self.ignore_stat_effects,
            conditional_modifiers: self.conditional_modifiers,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum ModifiableSkillEffectType {
    FlatDamage {
        damage: ModifiableDamageMap,
        crit_chance: ModifiableChance,
        crit_damage: ModifiableValue<f64>,
    },
    ApplyStatus {
        statuses: Vec<ModifiableApplyStatusEffect>,
        duration: ModifiableChanceRange<ModifiableValue<f64>>,
    },
    Restore {
        restore_type: RestoreType,
        value: ModifiableChanceRange<ModifiableValue<f64>>,
        modifier: Modifier,
    },
    Resurrect,
}

impl ModifiableSkillEffectType {
    fn evaluate(self) -> SkillEffectType {
        use SkillEffectType::*;
        match self {
            ModifiableSkillEffectType::FlatDamage {
                damage,
                crit_chance,
                crit_damage,
            } => FlatDamage {
                damage: damage.iter().map(|(x,y)| (*x, y.evaluate())).collect(),
                crit_chance: crit_chance.evaluate(),
                crit_damage: crit_damage.evaluate(),
            },
            ModifiableSkillEffectType::ApplyStatus { statuses, duration } => ApplyStatus {
                statuses: statuses.iter().map(|s| s.evaluate()).collect(),
                duration: duration.evaluate(),
            },
            ModifiableSkillEffectType::Restore {
                restore_type,
                value,
                modifier,
            } => Restore {
                restore_type,
                value: value.evaluate(),
                modifier,
            },
            ModifiableSkillEffectType::Resurrect => Resurrect,
        }
    }
}

impl From<&SkillEffectType> for ModifiableSkillEffectType {
    fn from(value: &SkillEffectType) -> Self {
        use ModifiableSkillEffectType::*;
        match value {
            SkillEffectType::FlatDamage {
                damage,
                crit_chance,
                crit_damage,
            } => FlatDamage {
                damage: to_modifiable_damage_map(damage),
                crit_chance: crit_chance.into(),
                crit_damage: (*crit_damage).into(),
            },
            SkillEffectType::ApplyStatus { statuses, duration } => ApplyStatus {
                statuses: statuses.iter().map(|s| s.into()).collect(),
                duration: (*duration).into(),
            },
            SkillEffectType::Restore {
                restore_type,
                value,
                modifier,
            } => Restore {
                restore_type: *restore_type,
                value: (*value).into(),
                modifier: *modifier,
            },
            SkillEffectType::Resurrect => Resurrect,
        }
    }
}

impl From<&ModifiableSkillEffectType> for StatSkillEffectType {
    fn from(value: &ModifiableSkillEffectType) -> Self {
        use StatSkillEffectType::*;
        match value {
            ModifiableSkillEffectType::FlatDamage { .. } => FlatDamage {},
            ModifiableSkillEffectType::ApplyStatus { statuses, .. } => ApplyStatus {
                status_type: statuses.first().map(|status| (&status.status_type).into()),
            },
            ModifiableSkillEffectType::Restore { restore_type, .. } => Restore {
                restore_type: Some(*restore_type),
            },
            ModifiableSkillEffectType::Resurrect => Resurrect,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct ModifiableApplyStatusEffect {
    status_type: StatusSpecs,
    value: ModifiableChanceRange<ModifiableValue<f64>>,
    cumulate: bool,
    replace_on_value_only: bool,
}

impl From<&ApplyStatusEffect> for ModifiableApplyStatusEffect {
    fn from(value: &ApplyStatusEffect) -> Self {
        ModifiableApplyStatusEffect {
            status_type: value.status_type.clone(),
            value: value.value.into(),
            cumulate: value.cumulate,
            replace_on_value_only: value.replace_on_value_only,
        }
    }
}

impl ModifiableApplyStatusEffect{
    fn evaluate(&self) -> ApplyStatusEffect {
        ApplyStatusEffect{
            status_type: self.status_type.clone(),
            value: self.value.evaluate(),
            cumulate: self.cumulate,
            replace_on_value_only: self.replace_on_value_only,
        }
    }
}