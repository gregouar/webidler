use anyhow::{anyhow, Result};

use axum::{extract::State, middleware, routing::post, Extension, Json, Router};

use shared::{
    data::market::MarketItem,
    http::{
        client::{
            BrowseMarketItemsRequest, BuyMarketItemRequest, EditMarketItemRequest,
            RejectMarketItemRequest, SellMarketItemRequest,
        },
        server::{
            BrowseMarketItemsResponse, BuyMarketItemResponse, EditMarketItemResponse,
            RejectMarketItemResponse, SellMarketItemResponse,
        },
    },
};

use crate::{
    app_state::{AppState, MasterStore},
    auth::{self, CurrentUser},
    db::{self, market::MarketEntry},
    game::{
        data::{inventory_data::inventory_data_to_player_inventory, items_store::ItemsStore},
        systems::{inventory_controller, items_controller, stashes_controller},
    },
    rest::utils::{verify_character_in_town, verify_character_user},
};

use super::AppError;

pub fn routes(app_state: AppState) -> Router<AppState> {
    let auth_routes = Router::new()
        .route("/market/buy", post(post_buy_market_item))
        .route("/market/reject", post(post_reject_market_item))
        .route("/market/sell", post(post_sell_market_item))
        .route("/market/edit", post(post_edit_market_item))
        .layer(middleware::from_fn_with_state(
            app_state,
            auth::authorization_middleware,
        ));

    Router::new()
        .route("/market", post(post_browse_market))
        .merge(auth_routes)
}

pub async fn post_browse_market(
    State(db_pool): State<db::DbPool>,
    State(master_store): State<MasterStore>,
    Json(payload): Json<BrowseMarketItemsRequest>,
) -> Result<Json<BrowseMarketItemsResponse>, AppError> {
    let (items, has_more) = db::market::read_market_items(
        &db_pool,
        &payload.character_id,
        payload.own_listings,
        payload.is_deleted,
        payload.filters,
        payload.skip as i64,
        payload.limit.into_inner(),
    )
    .await?;

    Ok(Json(BrowseMarketItemsResponse {
        items: items
            .into_iter()
            .filter_map(|market_item_entry| {
                into_market_item(&master_store.items_store, market_item_entry)
            })
            .collect(),
        has_more,
    }))
}

pub async fn post_buy_market_item(
    State(db_pool): State<db::DbPool>,
    State(master_store): State<MasterStore>,
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<BuyMarketItemRequest>,
) -> Result<Json<BuyMarketItemResponse>, AppError> {
    let mut tx = db_pool.begin().await?;

    let character = db::characters::read_character(&mut *tx, &payload.character_id)
        .await?
        .ok_or(AppError::NotFound)?;

    verify_character_user(&character, &current_user)?;
    verify_character_in_town(&character)?;

    let market_buy_entry = db::market::buy_item(
        &mut tx,
        payload.item_index as i64,
        Some(payload.character_id),
    )
    .await?
    .ok_or(AppError::NotFound)?;

    let item_bought = stashes_controller::take_stash_item(
        &mut tx,
        &master_store.items_store,
        None,
        market_buy_entry.stash_item_id,
    )
    .await?;

    if let Some(recipient_id) = market_buy_entry.recipient_id {
        if recipient_id != character.character_id && character.user_id != item_bought.user_id
        // Allow seller to remove own listing
        {
            return Err(AppError::Forbidden);
        }
    }

    // TODO: Move check to equip, allow to buy/exchange higher level items
    // if character.max_area_level < item_bought.item_level as i32 {
    //     return Err(AppError::UserError("character level too low".to_string()));
    // }

    if let Some(character_id) = item_bought.character_id {
        db::characters::update_character_resources(
            &mut *tx,
            &character_id,
            market_buy_entry.price,
            0.0,
            0.0,
        )
        .await?;
    }
    // TODO: Add gems to stash otherwise

    let character_resources = db::characters::update_character_resources(
        &mut *tx,
        &payload.character_id,
        -market_buy_entry.price,
        0.0,
        0.0,
    )
    .await?;

    if character_resources.resource_gems < 0.0 {
        return Err(AppError::UserError("not enough gems".into()));
    }

    let (inventory_data, _, _) =
        db::characters_data::load_character_data(&mut *tx, &payload.character_id)
            .await?
            .ok_or(anyhow!("inventory not found"))?;

    let mut inventory =
        inventory_data_to_player_inventory(&master_store.items_store, inventory_data);

    inventory_controller::store_item_to_bag(&mut inventory, item_bought.item_specs)?;

    db::characters_data::save_character_inventory(&mut *tx, &payload.character_id, &inventory)
        .await?;

    tx.commit().await?;

    Ok(Json(BuyMarketItemResponse {
        resource_gems: character_resources.resource_gems,
        inventory,
    }))
}

pub async fn post_reject_market_item(
    State(db_pool): State<db::DbPool>,
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<RejectMarketItemRequest>,
) -> Result<Json<RejectMarketItemResponse>, AppError> {
    let character = db::characters::read_character(&db_pool, &payload.character_id)
        .await?
        .ok_or(AppError::NotFound)?;

    verify_character_user(&character, &current_user)?;

    if !db::market::reject_item(&db_pool, payload.item_index as i64, &payload.character_id).await? {
        return Err(AppError::NotFound);
    }

    Ok(Json(RejectMarketItemResponse {}))
}

pub async fn post_sell_market_item(
    State(db_pool): State<db::DbPool>,
    State(master_store): State<MasterStore>,
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<SellMarketItemRequest>,
) -> Result<Json<SellMarketItemResponse>, AppError> {
    let mut tx = db_pool.begin().await?;

    let character = db::characters::read_character(&mut *tx, &payload.character_id)
        .await?
        .ok_or(AppError::NotFound)?;

    verify_character_user(&character, &current_user)?;
    verify_character_in_town(&character)?;

    let stash = db::stashes::get_character_market_stash(&mut *tx, &payload.character_id)
        .await?
        .ok_or(AppError::NotFound)?;

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

    let stash_item_id =
        stashes_controller::store_stash_item(&mut tx, &payload.character_id, &stash, &item_specs)
            .await?;

    db::market::sell_item(
        &mut tx,
        &stash_item_id,
        recipient_id,
        payload.price,
        (&item_specs).try_into()?,
    )
    .await?;

    db::characters_data::save_character_inventory(&mut *tx, &payload.character_id, &inventory)
        .await?;

    tx.commit().await?;

    Ok(Json(SellMarketItemResponse { inventory }))
}

pub async fn post_edit_market_item(
    State(db_pool): State<db::DbPool>,
    State(master_store): State<MasterStore>,
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<EditMarketItemRequest>,
) -> Result<Json<EditMarketItemResponse>, AppError> {
    let mut tx = db_pool.begin().await?;

    let character = db::characters::read_character(&mut *tx, &payload.character_id)
        .await?
        .ok_or(AppError::NotFound)?;

    verify_character_user(&character, &current_user)?;
    verify_character_in_town(&character)?;

    let market_item = db::market::buy_item(
        &mut tx,
        payload.item_index as i64,
        Some(character.character_id),
    )
    .await?
    .ok_or(AppError::NotFound)?;

    let item = stashes_controller::read_stash_item(
        &mut tx,
        &master_store.items_store,
        market_item.stash_item_id,
    )
    .await?;

    if item.user_id != character.user_id {
        return Err(AppError::Forbidden);
    }

    db::market::sell_item(
        &mut tx,
        &market_item.stash_item_id,
        market_item.recipient_id,
        payload.price,
        (&item.item_specs).try_into()?,
    )
    .await?;

    tx.commit().await?;

    Ok(Json(EditMarketItemResponse {}))
}

fn into_market_item(items_store: &ItemsStore, market_entry: MarketEntry) -> Option<MarketItem> {
    Some(MarketItem {
        item_id: market_entry.market_id as usize,
        owner_id: market_entry.character_id,
        owner_name: market_entry.character_name,
        recipient: market_entry.recipient_id.map(|recipient_id| {
            (
                recipient_id,
                market_entry.recipient_name.unwrap_or_default(),
            )
        }),
        rejected: market_entry.rejected,
        price: market_entry.price,

        item_specs: items_controller::init_item_specs_from_store(
            items_store,
            serde_json::from_value(market_entry.item_data).ok()?,
        )?,

        created_at: market_entry.created_at.into(),

        deleted_at: market_entry.deleted_at.map(Into::into),
        deleted_by: market_entry.deleted_by_id.map(|deleted_by_id| {
            (
                deleted_by_id,
                market_entry.deleted_by_name.unwrap_or_default(),
            )
        }),
    })
}
