use anyhow::{anyhow, Context, Result};

use axum::{
    extract::{Path, State},
    middleware,
    routing::post,
    Extension, Json, Router,
};

use shared::{
    constants::{MAX_MARKET_PRIVATE_LISTINGS, MAX_MARKET_PUBLIC_LISTINGS},
    data::stash::{StashId, StashItem, StashType},
    http::{
        client::{BrowseStashItemsRequest, StoreStashItemRequest},
        server::{BrowseStashItemsResponse, StoreStashItemResponse, TakeStashItemResponse},
    },
};

use crate::{
    app_state::{AppState, MasterStore},
    auth::{self, CurrentUser},
    db::{self, stash_items::StashEntry},
    game::{
        data::{inventory_data::inventory_data_to_player_inventory, items_store::ItemsStore},
        systems::items_controller,
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

fn verify_stash_access(current_user: &CurrentUser, stash: &Stash) -> Result<(), AppError> {
    if !match stash.stash_type {
        StashType::Market => true,
        StashType::User => stash.user_id == current_user.user_details.user.user_id,
    } {
        return AppError::Forbidden;
    }
}

pub async fn post_browse_stash(
    State(db_pool): State<db::DbPool>,
    State(master_store): State<MasterStore>,
    Extension(current_user): Extension<CurrentUser>,
    Path(stash_id): Path<StashId>,
    Json(payload): Json<BrowseStashItemsRequest>,
) -> Result<Json<BrowseStashItemsResponse>, AppError> {
    let stash = db::stashes::read_stash(stash_id).await?;

    verify_stash_access(&current_user, stash)?;

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
            .filter_map(|item_entry| into_stash_item(&master_store.items_store, item_entry))
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

    let character = db::characters::read_character(&mut *tx, &payload.character_id)
        .await?
        .ok_or(AppError::NotFound)?;

    verify_stash_access(&current_user, stash)?;
    verify_character_user(&character, &current_user)?;
    verify_character_in_town(&character)?;

    let (inventory_data, _, _) =
        db::characters_data::load_character_data(&mut *tx, &payload.character_id)
            .await?
            .ok_or(AppError::UserError("newbies can't buy items".into()))?;

    let mut inventory =
        inventory_data_to_player_inventory(&master_store.items_store, inventory_data);

    if inventory.bag.len() >= inventory.max_bag_size as usize {
        return Err(AppError::UserError("not enough space".into()));
    }

    let item_bought = db::market::buy_item(
        &mut tx,
        payload.item_index as i64,
        Some(payload.character_id),
    )
    .await?
    .ok_or(AppError::NotFound)?;

    if let Some(recipient_id) = item_bought.recipient_id {
        if recipient_id != character.character_id
            && character.character_id != item_bought.character_id
        // Allow seller to remove own listing
        {
            return Err(AppError::Forbidden);
        }
    }

    if character.max_area_level < item_bought.item_level as i32 {
        return Err(AppError::UserError("character level too low".to_string()));
    }

    db::characters::update_character_resources(
        &mut *tx,
        &item_bought.character_id,
        item_bought.price,
        0.0,
        0.0,
    )
    .await?;

    let character_resources = db::characters::update_character_resources(
        &mut *tx,
        &payload.character_id,
        -item_bought.price,
        0.0,
        0.0,
    )
    .await?;

    if character_resources.resource_gems < 0.0 {
        return Err(AppError::UserError("not enough gems".into()));
    }

    inventory.bag.push(
        items_controller::init_item_specs_from_store(
            &master_store.items_store,
            serde_json::from_value(item_bought.item_data).context("invalid data")?,
        )
        .ok_or(anyhow!("base item not found"))?,
    );

    db::characters_data::save_character_inventory(&mut *tx, &payload.character_id, &inventory)
        .await?;

    tx.commit().await?;

    Ok(Json(TakeStashItemResponse {
        resource_gems: character_resources.resource_gems,
        inventory,
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

    let character = db::characters::read_character(&mut *tx, &payload.character_id)
        .await?
        .ok_or(AppError::NotFound)?;

    verify_stash_access(&current_user, stash)?;
    verify_character_user(&character, &current_user)?;
    verify_character_in_town(&character)?;

    let (public_listings, private_listings) =
        db::market::count_market_items(&mut *tx, &payload.character_id).await?;

    if payload.recipient_name.is_none() {
        if public_listings >= MAX_MARKET_PUBLIC_LISTINGS {
            return Err(AppError::UserError(format!(
                "too many public listings (max {MAX_MARKET_PUBLIC_LISTINGS})"
            )));
        }
    } else if private_listings >= MAX_MARKET_PRIVATE_LISTINGS {
        return Err(AppError::UserError(format!(
            "too many private offers (max {MAX_MARKET_PRIVATE_LISTINGS})"
        )));
    }

    let (inventory_data, _, _) =
        db::characters_data::load_character_data(&mut *tx, &payload.character_id)
            .await?
            .ok_or(anyhow!("inventory not found"))?;

    let mut inventory =
        inventory_data_to_player_inventory(&master_store.items_store, inventory_data);

    let item_specs = (payload.item_index < inventory.bag.len())
        .then(|| inventory.bag.remove(payload.item_index))
        .ok_or(AppError::NotFound)?;

    let recipient_id = if let Some(character_name) = payload.recipient_name {
        let character_name = character_name.into_inner();
        Some(
            db::characters::get_character_by_name(&mut *tx, &character_name)
                .await?
                .ok_or(AppError::UserError(format!(
                    "character '{}' not found",
                    character_name
                )))?,
        )
    } else {
        None
    };

    if recipient_id.unwrap_or_default() == character.character_id {
        return Err(AppError::UserError("cannot offer to yourself".into()));
    }

    db::market::sell_item(
        &mut tx,
        &payload.character_id,
        recipient_id,
        payload.price,
        &item_specs,
    )
    .await?;

    db::characters_data::save_character_inventory(&mut *tx, &payload.character_id, &inventory)
        .await?;

    tx.commit().await?;

    Ok(Json(StoreStashItemResponse { inventory }))
}

fn into_market_item(items_store: &ItemsStore, item_entry: StashEntry) -> Option<StashItem> {
    Some(StashItem {
        stash_id: item_entry.stash_id,
        stash_item_id: item_entry.stash_item_id as usize,
        character_id: item_entry.character_id,
        character_name: item_entry.character_name,

        item_specs: items_controller::init_item_specs_from_store(
            items_store,
            serde_json::from_value(item_entry.item_data).ok()?,
        )?,

        created_at: item_entry.created_at.into(),
    })
}
