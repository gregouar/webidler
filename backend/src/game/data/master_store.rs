use std::sync::Arc;

use super::items_table::ItemsTable;

pub struct MasterData {
    items_table: Arc<ItemsTable>,
}
