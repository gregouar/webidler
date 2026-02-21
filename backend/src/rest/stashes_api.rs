use anyhow::{Result, anyhow};

use axum::{
    Extension, Json, Router,
    extract::{Path, State},
    middleware,
    routing::post,
};

use shared::{
    computations,
    data::stash::{Stash, StashId, StashType},
    http::{
        client::{
            BrowseStashItemsRequest, ExchangeGemsStashRequest, StashAction, StoreStashItemRequest,
            TakeStashItemRequest, UpgradeStashRequest,
        },
        server::{
            BrowseStashItemsResponse, ExchangeGemsStashResponse, StoreStashItemResponse,
            TakeStashItemResponse, UpgradeStashResponse,
        },
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
        .route("/stashes/upgrade", post(post_upgrade_stash))
        .route("/stashes/{stash_id}", post(post_browse_stash))
        .route("/stashes/{stash_id}/gems", post(post_exchange_gems))
        .route("/stashes/{stash_id}/take", post(post_take_stash_item))
        .route("/stashes/{stash_id}/store", post(post_store_stash_item))
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

pub async fn post_upgrade_stash(
    State(db_pool): State<db::DbPool>,
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<UpgradeStashRequest>,
) -> Result<Json<UpgradeStashResponse>, AppError> {
    let mut tx = db_pool.begin().await?;

    let character = db::characters::read_character(&mut *tx, &payload.character_id)
        .await?
        .ok_or(AppError::NotFound)?;

    verify_character_user(&character, &current_user)?;
    verify_character_in_town(&character)?;

    let stash = match db::stashes::get_character_stash_by_type(
        &mut *tx,
        &payload.character_id,
        payload.stash_type,
    )
    .await?
    {
        Some(stash) => stash,
        None => {
            db::stashes::create_stash(
                &mut *tx,
                character.user_id,
                payload.stash_type,
                0,
                "New stash",
            )
            .await?
        }
    };

    verify_stash_access_write(&current_user, &stash)?;

    let mut stash = stash.into();
    let (max_items, cost) = computations::stash_upgrade(&stash);
    stash.max_items = max_items;

    let character_resources = db::characters::update_character_resources(
        &mut *tx,
        &payload.character_id,
        0.0,
        0.0,
        -cost,
    )
    .await?;

    if character_resources.resource_gold < 0.0 {
        return Err(AppError::UserError("not enough gold".to_string()));
    }

    db::stashes::update_stash_size(&mut *tx, &stash.stash_id, max_items).await?;

    tx.commit().await?;

    Ok(Json(UpgradeStashResponse {
        resource_gold: character_resources.resource_gold,
        stash,
    }))
}

pub async fn post_exchange_gems(
    State(db_pool): State<db::DbPool>,
    Extension(current_user): Extension<CurrentUser>,
    Path(stash_id): Path<StashId>,
    Json(payload): Json<ExchangeGemsStashRequest>,
) -> Result<Json<ExchangeGemsStashResponse>, AppError> {
    let mut tx = db_pool.begin().await?;

    let character = db::characters::read_character(&mut *tx, &payload.character_id)
        .await?
        .ok_or(AppError::NotFound)?;

    verify_character_user(&character, &current_user)?;
    verify_character_in_town(&character)?;

    let mut stash = db::stashes::get_stash(&db_pool, &stash_id)
        .await?
        .ok_or(AppError::NotFound)?;

    verify_stash_access_write(&current_user, &stash)?;

    let gems_amount = payload.amount.into_inner();
    let gems_difference = match payload.stash_action {
        StashAction::Store => gems_amount.min(character.resource_gems),
        StashAction::Take => -gems_amount.min(stash.resource_gems),
    };

    stash.resource_gems =
        db::stashes::update_stash_gems(&mut *tx, &stash_id, gems_difference).await?;

    let character_resources = db::characters::update_character_resources(
        &mut *tx,
        &payload.character_id,
        -gems_difference,
        0.0,
        0.0,
    )
    .await?;

    tx.commit().await?;

    Ok(Json(ExchangeGemsStashResponse {
        resource_gems: character_resources.resource_gems,
        stash: stash.into(),
    }))
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
