use shared::{
    constants::{MAX_AREA_LEVEL, MAX_POWER_SHARD_LEVEL_BASE},
    data::{
        area::{AreaLevel, AreaSpecs, AreaState},
        chance::{Chance, ChanceRange},
        item::{ItemSpecs, SkillRange, SkillShape},
        item_affix::{AffixEffectScope, aggregate_effects},
        modifier::{ModifiableValue, Modifier},
        skill::{SkillEffect, SkillEffectType, SkillType},
        stat_effect::{EffectsMap, StatType, StatusOnHitValueType},
        trigger::{
            EventTrigger, HitTrigger, TriggerEffect, TriggerEffectModifier,
            TriggerEffectModifierSource, TriggerTarget,
        },
        values::{Damage, Percent},
    },
};

use crate::game::data::{
    area::AreaBlueprint,
    items_store::ItemsStore,
    master_store::{LootTablesStore, StatusesStore},
};

pub fn decrease_area_level(area_state: &mut AreaState, amount: i32) {
    area_state.area_level = (area_state.area_level as i32)
        .saturating_sub(amount)
        .clamp(1, MAX_AREA_LEVEL as i32) as AreaLevel;
    area_state.used_wave_indices.clear();
    area_state.waves_done = 1;
}

pub fn auto_increase_area_level(area_state: &mut AreaState, amount: i32) {
    if area_state.auto_progress {
        area_state.area_level = (area_state.area_level as i32)
            .saturating_add(amount)
            .clamp(1, MAX_AREA_LEVEL as i32) as AreaLevel;
    }
    area_state.used_wave_indices.clear();
    area_state.waves_done = 1;
}

pub fn init_area_specs(
    items_store: &ItemsStore,
    loot_tables_store: &LootTablesStore,
    statuses_store: &StatusesStore,
    area_blueprint: &mut AreaBlueprint,
    map_item: &Option<ItemSpecs>,
) -> AreaSpecs {
    let mut area_specs = area_blueprint.specs.clone();

    let map_effects = if let Some(map_item) = map_item {
        if let Some(map_specs) = &map_item.map_specs {
            area_specs.reward_picks += map_specs.reward_picks;
            area_specs.reward_slots += map_specs.reward_slots;

            // for loot_table_id in map_specs.loot_tables.iter() {
            //     if let Some(loot_table) = loot_tables_store.get(loot_table_id) {
            //         if loot_table.area_specific {
            //             area_blueprint
            //                 .loot_table_area
            //                 .entries
            //                 .extend(loot_table.entries.iter().cloned());
            //         } else {
            //             area_blueprint
            //                 .loot_table
            //                 .entries
            //                 .extend(loot_table.entries.iter().cloned());
            //         }
            //     }
            // }

            area_blueprint.reward_loot_table = map_specs
                .reward_loot_table
                .as_ref()
                .and_then(|loot_table_id| loot_tables_store.get(loot_table_id))
                .cloned();
        }

        if let Ok(map_item_base) = items_store.get_item_base(map_item) {
            for trigger_specs in map_item_base.triggers.iter() {
                area_specs.triggers.push(
                    trigger_specs.trigger.clone(),
                    trigger_specs.trigger_effect.clone(),
                    None,
                );
            }
        }

        EffectsMap::combine_all(
            std::iter::once(aggregate_effects(
                &map_item.affixes,
                AffixEffectScope::Local,
                false,
            ))
            .chain(std::iter::once(aggregate_effects(
                &map_item.affixes,
                AffixEffectScope::Global,
                false,
            ))),
        )
    } else {
        Default::default()
    };

    // TODO: Proper aggregation of on hit stats to cumulate
    for effect in map_effects
        .iter()
        .filter(|effect| effect.modifier == Modifier::Flat)
    {
        if let StatType::RestoreOnHit { .. } = &effect.stat {
            // TODO
        }

        if let StatType::StatusOnHit {
            status_id,
            value_type,
            skill_type,
            item_slot: _,
        } = &effect.stat
        {
            let Some(status_specs) = statuses_store.get(status_id) else {
                continue;
            };
            let value = ModifiableValue::from(Damage::new(effect.value));
            let status_value = if *value_type == StatusOnHitValueType::Power {
                value
            } else {
                ModifiableValue::from(Damage::new(100.0))
            };
            let (modifiers, status_value) = if status_specs.damage_type.is_some() {
                (
                    [TriggerEffectModifier {
                        stat: StatType::Damage {
                            skill_filter: Default::default(),
                            damage_type: Default::default(),
                            min_max: Default::default(),
                            is_hit: Default::default(),
                        },
                        modifier: Modifier::Flat,
                        factor: status_value.get() * 0.01,
                        source: TriggerEffectModifierSource::HitDamage(None),
                        hidden: false,
                    }]
                    .into(),
                    ModifiableValue::from(Damage::new(0.0)),
                )
            } else {
                (Vec::new(), status_value)
            };

            area_specs.triggers.push(
                EventTrigger::OnHit(HitTrigger {
                    skill_type: *skill_type,
                    is_hurt: Some(true),
                    is_triggered: Some(false),
                    ..Default::default()
                }),
                TriggerEffect {
                    trigger_id: format!("status_on_hit_{:?}_{:?}", status_id, skill_type),
                    target: TriggerTarget::SameTarget,
                    skill_range: SkillRange::Any,
                    skill_type: SkillType::Other,
                    skill_shape: SkillShape::Single,
                    skill_repeat: Default::default(),
                    modifiers,
                    effects: vec![SkillEffect {
                        success_chance: if *value_type == StatusOnHitValueType::Chance {
                            Chance {
                                value: ModifiableValue::new(
                                    Percent::new(value.base().get() as f32),
                                    value.increased(),
                                    value.decreased(),
                                    value.more(),
                                    value.converted(),
                                ),
                                lucky_chance: Default::default(),
                            }
                        } else {
                            Chance::new_sure()
                        },
                        effect_type: SkillEffectType::ApplyStatus {
                            status_id: status_id.clone(),
                            value: ChanceRange::from(status_value),
                            value_factor: 1.0,
                            duration: None,
                            escalation: None,
                            max_stacks: None,
                            avoidable: None,
                            replace_on_value_only: false,
                            computed_status_triggers: None,
                        },
                        ignore_stat_effects: [StatType::Damage {
                            skill_filter: Default::default(),
                            damage_type: Default::default(),
                            min_max: Default::default(),
                            is_hit: Some(true),
                        }]
                        .into(),
                        conditional_modifiers: Default::default(),
                        independent_application: false,
                        optional_application: false,
                        description: None,
                        as_skill_type: None,
                    }],
                    trigger_propagate: false,
                },
                None,
            )
        }
    }

    area_specs.effects = EffectsMap::combine_all(
        std::iter::once(map_effects).chain(std::iter::once(area_specs.effects)),
    );

    area_specs.max_power_shard_level = map_item
        .as_ref()
        .and_then(|map_item| map_item.map_specs.as_ref())
        .and_then(|map_specs| map_specs.max_power_shard_level)
        .unwrap_or(MAX_POWER_SHARD_LEVEL_BASE);

    let item_area_chance = compute_area_specs(&mut area_specs);

    for entry in area_blueprint.loot_table_area.entries.iter_mut() {
        entry.weight = (entry.weight as f64 * item_area_chance * 0.01)
            .round()
            .max(0.0) as u64;
    }
    area_blueprint
        .loot_table
        .entries
        .append(&mut area_blueprint.loot_table_area.entries);

    area_specs
}

fn compute_area_specs(area_specs: &mut AreaSpecs) -> f64 {
    let mut item_area_chance: ModifiableValue<f64> = 100.0.into();

    for effect in area_specs.effects.iter() {
        match effect.stat {
            StatType::ItemRarity => area_specs.loot_rarity.apply_effect(&effect),
            StatType::ItemAreaChance => item_area_chance.apply_effect(&effect),
            StatType::ItemLevel => area_specs.item_level_modifier.apply_effect(&effect),
            StatType::GemsFind => area_specs.gems_find.apply_effect(&effect),
            StatType::GoldFind => area_specs.gold_find.apply_effect(&effect),
            StatType::PowerLevel => area_specs.power_level.apply_effect(&effect),
            _ => {}
        }
    }

    *item_area_chance
}
