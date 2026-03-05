use shared::data::{
    area::{AreaLevel, AreaSpecs, AreaState},
    item::ItemSpecs,
    item_affix::AffixEffectScope,
    stat_effect::{EffectsMap, StatType},
};

use crate::game::data::{area::AreaBlueprint, master_store::LootTablesStore};

pub fn decrease_area_level(area_state: &mut AreaState, amount: AreaLevel) {
    area_state.area_level = area_state.area_level.saturating_sub(amount).max(1);
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
                    area_blueprint
                        .loot_table
                        .entries
                        .extend(loot_table.entries.iter().cloned());
                }
            }
        }

        area_specs.triggers.extend(
            map_item
                .base
                .triggers
                .iter()
                .map(|trigger_specs| trigger_specs.triggered_effect.clone()),
        );

        EffectsMap::combine_all(
            std::iter::once(
                map_item
                    .modifiers
                    .aggregate_effects(AffixEffectScope::Local),
            )
            .chain(std::iter::once(
                map_item
                    .modifiers
                    .aggregate_effects(AffixEffectScope::Global),
            )),
        )
    } else {
        Default::default()
    };

    area_specs.effects = EffectsMap::combine_all(
        std::iter::once(map_effects).chain(std::iter::once(area_specs.effects)),
    );

    compute_area_specs(&mut area_specs);

    area_specs
}

fn compute_area_specs(area_specs: &mut AreaSpecs) {
    for effect in area_specs.effects.iter() {
        match effect.stat {
            StatType::ItemRarity => area_specs.loot_rarity.apply_effect(&effect),
            StatType::ItemLevel => area_specs.item_level_modifier.apply_effect(&effect),
            StatType::GemsFind => area_specs.gems_find.apply_effect(&effect),
            _ => {}
        }
    }
}
