use shared::data::{
    area::{AreaLevel, AreaSpecs, AreaState},
    stat_effect::{StatEffect, StatType},
};

use crate::game::utils::modifiable_value::ModifiableValue;

pub fn decrease_area_level(world_specs: &AreaSpecs, area_state: &mut AreaState, amount: AreaLevel) {
    area_state.area_level = area_state
        .area_level
        .saturating_sub(amount)
        .max(1)
        .max(world_specs.starting_level);
    area_state.waves_done = 1;
}

pub fn compute_area_state(area_state: &mut AreaState, effects: &[StatEffect]) {
    let mut loot_rarity: ModifiableValue<_> = area_state.loot_rarity.into();

    for effect in effects.iter() {
        if effect.stat == StatType::ItemRarity { loot_rarity.apply_effect(effect) }
    }

    area_state.loot_rarity = loot_rarity.evaluate();
}
