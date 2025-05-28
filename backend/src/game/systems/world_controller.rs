use shared::data::world::{AreaLevel, WorldState};

pub fn decrease_area_level(world_state: &mut WorldState, amount: AreaLevel) {
    world_state.area_level = world_state.area_level.saturating_sub(amount).max(1);
    world_state.waves_done = 1;
}
