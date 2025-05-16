use shared::data::world::{AreaLevel, WorldState};

pub fn decrease_area_level(world_state: &mut WorldState, amount: AreaLevel) {
    world_state.area_level = world_state
        .area_level
        .checked_sub(amount)
        .unwrap_or(1)
        .max(1);
    world_state.waves_done = 0;
}
