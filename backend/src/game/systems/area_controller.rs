use shared::data::{
    area::{AreaLevel, AreaSpecs, AreaState},
    stat_effect::{StatEffect, StatType},
};

pub fn decrease_area_level(world_specs: &AreaSpecs, area_state: &mut AreaState, amount: AreaLevel) {
    area_state.area_level = area_state
        .area_level
        .saturating_sub(amount)
        .max(1)
        .max(world_specs.starting_level);
    area_state.waves_done = 1;
}

pub fn compute_area_state(area_state: &mut AreaState, effects: &[StatEffect]) {
    for effect in effects.iter() {
        if effect.stat == StatType::ItemRarity { area_state.loot_rarity.apply_effect(effect) }
    }
}
