use shared::data::area::{AreaLevel, AreaSpecs, AreaState};

pub fn decrease_area_level(world_specs: &AreaSpecs, area_state: &mut AreaState, amount: AreaLevel) {
    area_state.area_level = area_state
        .area_level
        .saturating_sub(amount)
        .max(1)
        .max(world_specs.starting_level);
    area_state.waves_done = 0;
}
