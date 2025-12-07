use anyhow::{anyhow, Context, Result};
use sqlx::Transaction;

use shared::data::{item::ItemSpecs, stash::StashId, user::UserCharacterId};

use crate::{
    db::{self, pool::Database, stash_items::StashItemId, stashes::StashEntry},
    game::{data::items_store::ItemsStore, systems::items_controller},
    rest::AppError,
};

pub async fn take_stash_item<'c>(
    executor: &mut Transaction<'c, Database>,
    items_store: &ItemsStore,
    stash_id: &StashId,
    stash_item_id: StashItemId,
) -> Result<ItemSpecs, AppError> {
    let stash_item_entry = db::stash_items::take_item(&mut *executor, stash_id, stash_item_id)
        .await?
        .ok_or(AppError::NotFound)?;

    Ok(items_controller::init_item_specs_from_store(
        &items_store,
        serde_json::from_value(stash_item_entry.item_data).context("invalid data")?,
    )
    .ok_or(anyhow!("base item not found"))?)
}

pub async fn store_stash_item<'c>(
    executor: &mut Transaction<'c, Database>,
    character_id: &UserCharacterId,
    stash: &StashEntry,
    item_specs: &ItemSpecs,
) -> Result<StashItemId, AppError> {
    if db::stash_items::count_stash_items(&mut **executor, &stash.stash_id).await?
        >= stash.max_items
    {
        return Err(AppError::UserError(format!("stash if full")));
    }

    Ok(
        db::stash_items::store_item(&mut *executor, &stash.stash_id, character_id, item_specs)
            .await?,
    )
}
