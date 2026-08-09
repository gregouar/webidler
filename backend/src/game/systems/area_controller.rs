use shared::{
    constants::{MAX_AREA_LEVEL, MAX_POWER_SHARD_LEVEL_BASE},
    data::{
        area::{AreaLevel, AreaSpecs, AreaState},
        item::ItemSpecs,
        item_affix::AffixEffectScope,
        modifier::ModifiableValue,
        stat_effect::{EffectsMap, StatType},
    },
};

use crate::game::data::{area::AreaBlueprint, master_store::LootTablesStore};

pub fn decrease_area_level(area_state: &mut AreaState, amount: i32) {
    area_state.area_level = (area_state.area_level as i32)
        .saturating_sub(amount)
        .clamp(1, MAX_AREA_LEVEL as i32) as AreaLevel;
    area_state.waves_done = 1;
}

pub fn init_area_specs(
    loot_tables_store: &LootTablesStore,
    area_blueprint: &mut AreaBlueprint,
    map_item: &Option<ItemSpecs>,
) -> AreaSpecs {
    let mut area_specs = area_blueprint.specs.clone();

    let map_effects = if let Some(map_item) = map_item {
        if let Some(map_specs) = &map_item.base.map_specs {
            area_specs.reward_picks += map_specs.reward_picks;
            area_specs.reward_slots += map_specs.reward_slots;

            for loot_table_id in map_specs.loot_tables.iter() {
                if let Some(loot_table) = loot_tables_store.get(loot_table_id) {
                    if loot_table.area_specific {
                        area_blueprint
                            .loot_table_area
                            .entries
                            .extend(loot_table.entries.iter().cloned());
                    } else {
                        area_blueprint
                            .loot_table
                            .entries
                            .extend(loot_table.entries.iter().cloned());
                    }
                }
            }

            area_blueprint.reward_loot_table = map_specs
                .reward_loot_table
                .as_ref()
                .and_then(|loot_table_id| loot_tables_store.get(loot_table_id))
                .cloned();
        }

        for trigger_specs in map_item.base.triggers.iter() {
            area_specs.triggers.push(
                trigger_specs.trigger.clone(),
                trigger_specs.trigger_effect.clone(),
                None,
            );
        }

        EffectsMap::combine_all(
            std::iter::once(
                map_item
                    .modifiers
                    .aggregate_effects(AffixEffectScope::Local, false),
            )
            .chain(std::iter::once(
                map_item
                    .modifiers
                    .aggregate_effects(AffixEffectScope::Global, false),
            )),
        )
    } else {
        Default::default()
    };

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
