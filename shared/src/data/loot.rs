use serde::{Deserialize, Serialize};

use super::item::ItemSpecs;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct QueuedLoot {
    pub identifier: u32,
    pub item_specs: ItemSpecs,
    pub state: LootState,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum LootState {
    Normal,
    WillDisappear,
    HasDisappeared,
}

impl Default for LootState {
    fn default() -> Self {
        LootState::Normal
    }
}
