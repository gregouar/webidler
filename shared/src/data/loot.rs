use serde::{Deserialize, Serialize};

use super::item::ItemSpecs;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct QueuedLoot {
    pub identifier: u32,
    pub item_specs: ItemSpecs,
    pub state: LootState,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Default)]
pub enum LootState {
    #[default]
    Normal,
    WillDisappear,
    HasDisappeared,
}
