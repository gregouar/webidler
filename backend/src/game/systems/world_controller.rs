use shared::data::world::{AreaLevel, WorldSpecs, WorldState};

pub fn decrease_area_level(
    world_specs: &WorldSpecs,
    world_state: &mut WorldState,
    amount: AreaLevel,
) {
    world_state.area_level = world_state
        .area_level
        .saturating_sub(amount)
        .max(1)
        .max(world_specs.starting_level);
    world_state.waves_done = 0;
}
