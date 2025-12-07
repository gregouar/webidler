use anyhow::{anyhow, Result};

use axum::{
    extract::{Path, State},
    middleware,
    routing::post,
    Extension, Json, Router,
};

use shared::{
    data::stash::{Stash, StashId, StashType},
    http::{
        client::{BrowseStashItemsRequest, StoreStashItemRequest, TakeStashItemRequest},
        server::{BrowseStashItemsResponse, StoreStashItemResponse, TakeStashItemResponse},
    },
};

use crate::{
    app_state::{AppState, MasterStore},
    auth::{self, CurrentUser},
    db::{self, stashes::StashEntry},
    game::{
        data::inventory_data::inventory_data_to_player_inventory,
        systems::{inventory_controller, stashes_controller},
    },
    rest::utils::{verify_character_in_town, verify_character_user},
};

use super::AppError;

pub fn routes(app_state: AppState) -> Router<AppState> {
    Router::new()
        .route("/stashes/{stash_id}", post(post_browse_stash))
        .route("/stashes/{stash_id}/buy", post(post_take_stash_item))
        .route("/stashes/{stash_id}/sell", post(post_store_stash_item))
        .layer(middleware::from_fn_with_state(
            app_state,
            auth::authorization_middleware,
        ))
}

fn verify_stash_access_write(
    current_user: &CurrentUser,
    stash: &StashEntry,
) -> Result<(), AppError> {
    if stash.user_id != current_user.user_details.user.user_id {
        return Err(AppError::Forbidden);
    }
    Ok(())
}

fn verify_stash_access_read(
    current_user: &CurrentUser,
    stash: &StashEntry,
) -> Result<(), AppError> {
    if !match stash.stash_type.0 {
        StashType::Market => true,
        StashType::User => stash.user_id == current_user.user_details.user.user_id,
    } {
        return Err(AppError::Forbidden);
    }
    Ok(())
}

pub async fn post_browse_stash(
    State(db_pool): State<db::DbPool>,
    State(master_store): State<MasterStore>,
    Extension(current_user): Extension<CurrentUser>,
    Path(stash_id): Path<StashId>,
    Json(payload): Json<BrowseStashItemsRequest>,
) -> Result<Json<BrowseStashItemsResponse>, AppError> {
    let stash = db::stashes::get_stash(&db_pool, &stash_id)
        .await?
        .ok_or(AppError::NotFound)?;

    verify_stash_access_read(&current_user, &stash)?;

    let (items, has_more) = db::stash_items::read_stash_items(
        &db_pool,
        stash_id,
        payload.filters,
        payload.skip as i64,
        payload.limit.into_inner(),
    )
    .await?;

    Ok(Json(BrowseStashItemsResponse {
        items: items
            .into_iter()
            .filter_map(|item_entry| {
                stashes_controller::into_stash_item(&master_store.items_store, item_entry)
            })
            .collect(),
        has_more,
    }))
}

pub async fn post_take_stash_item(
    State(db_pool): State<db::DbPool>,
    State(master_store): State<MasterStore>,
    Extension(current_user): Extension<CurrentUser>,
    Path(stash_id): Path<StashId>,
    Json(payload): Json<TakeStashItemRequest>,
) -> Result<Json<TakeStashItemResponse>, AppError> {
    let mut tx = db_pool.begin().await?;

    let mut stash = db::stashes::get_stash(&mut *tx, &stash_id)
        .await?
        .ok_or(AppError::NotFound)?;

    verify_stash_access_write(&current_user, &stash)?;

    let character = db::characters::read_character(&mut *tx, &payload.character_id)
        .await?
        .ok_or(AppError::NotFound)?;

    verify_character_user(&character, &current_user)?;
    verify_character_in_town(&character)?;

    let (inventory_data, _, _) =
        db::characters_data::load_character_data(&mut *tx, &payload.character_id)
            .await?
            .ok_or(AppError::UserError("newbies can't take items".into()))?;

    let mut inventory =
        inventory_data_to_player_inventory(&master_store.items_store, inventory_data);

    inventory_controller::store_item_to_bag(
        &mut inventory,
        stashes_controller::take_stash_item(
            &mut tx,
            &master_store.items_store,
            Some(&mut stash),
            payload.item_index as i64,
        )
        .await?
        .item_specs,
    )?;

    db::characters_data::save_character_inventory(&mut *tx, &payload.character_id, &inventory)
        .await?;

    tx.commit().await?;

    Ok(Json(TakeStashItemResponse {
        inventory,
        stash: stash.into(),
    }))
}

pub async fn post_store_stash_item(
    State(db_pool): State<db::DbPool>,
    State(master_store): State<MasterStore>,
    Extension(current_user): Extension<CurrentUser>,
    Path(stash_id): Path<StashId>,
    Json(payload): Json<StoreStashItemRequest>,
) -> Result<Json<StoreStashItemResponse>, AppError> {
    let mut tx = db_pool.begin().await?;

    let mut stash = db::stashes::get_stash(&mut *tx, &stash_id)
        .await?
        .ok_or(AppError::NotFound)?;

    verify_stash_access_write(&current_user, &stash)?;

    let character = db::characters::read_character(&mut *tx, &payload.character_id)
        .await?
        .ok_or(AppError::NotFound)?;

    verify_character_user(&character, &current_user)?;
    verify_character_in_town(&character)?;

    let (inventory_data, _, _) =
        db::characters_data::load_character_data(&mut *tx, &payload.character_id)
            .await?
            .ok_or(anyhow!("inventory not found"))?;

    let mut inventory =
        inventory_data_to_player_inventory(&master_store.items_store, inventory_data);

    let item_specs = (payload.item_index < inventory.bag.len())
        .then(|| inventory.bag.remove(payload.item_index))
        .ok_or(AppError::NotFound)?;

    stashes_controller::store_stash_item(&mut tx, &payload.character_id, &mut stash, &item_specs)
        .await?;

    db::characters_data::save_character_inventory(&mut *tx, &payload.character_id, &inventory)
        .await?;

    tx.commit().await?;

    Ok(Json(StoreStashItemResponse {
        inventory,
        stash: stash.into(),
    }))
}

impl From<db::stashes::StashEntry> for Stash {
    fn from(value: db::stashes::StashEntry) -> Self {
        Self {
            stash_id: value.stash_id,
            user_id: value.user_id,
            stash_type: value.stash_type.0,
            title: value.title,
            items_amount: value.items_amount as usize,
            max_items: value.max_items as usize,
            resource_gems: value.resource_gems,
        }
    }
}
