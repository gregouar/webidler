use anyhow::Result;
use sqlx::Transaction;

use shared::data::{
    item::ItemSpecs,
    stash::{StashId, StashItem},
    user::UserCharacterId,
};

use crate::{
    db::{
        self,
        pool::Database,
        stash_items::{StashItemEntry, StashItemId},
        stashes::StashEntry,
    },
    game::{data::items_store::ItemsStore, systems::items_controller},
    rest::AppError,
};

pub async fn read_stash_item<'c>(
    executor: &mut Transaction<'c, Database>,
    items_store: &ItemsStore,
    stash_item_id: StashItemId,
) -> Result<StashItem, AppError> {
    let stash_item_entry = db::stash_items::read_stash_item(&mut *executor, stash_item_id)
        .await?
        .ok_or(AppError::NotFound)?;

    into_stash_item(items_store, stash_item_entry).ok_or(AppError::NotFound)
}

pub async fn take_stash_item<'c>(
    executor: &mut Transaction<'c, Database>,
    items_store: &ItemsStore,
    stash_id: Option<&StashId>,
    stash_item_id: StashItemId,
) -> Result<StashItem, AppError> {
    let stash_item_entry = db::stash_items::take_item(&mut *executor, stash_id, stash_item_id)
        .await?
        .ok_or(AppError::NotFound)?;

    into_stash_item(items_store, stash_item_entry).ok_or(AppError::NotFound)
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

pub fn into_stash_item(items_store: &ItemsStore, item_entry: StashItemEntry) -> Option<StashItem> {
    Some(StashItem {
        stash_id: item_entry.stash_id,
        stash_item_id: item_entry.stash_item_id as usize,

        user_id: item_entry.user_id,
        character_id: item_entry.character_id,
        character_name: item_entry.character_name,

        item_specs: items_controller::init_item_specs_from_store(
            items_store,
            serde_json::from_value(item_entry.item_data).ok()?,
        )?,

        created_at: item_entry.created_at.into(),
    })
}
