use anyhow::{anyhow, Context, Result};

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
    constants::{MAX_MARKET_PRIVATE_LISTINGS, MAX_MARKET_PUBLIC_LISTINGS},
    db::{self, market::MarketEntry},
    game::{data::items_store::ItemsStore, systems::items_controller},
    rest::utils::{
        inventory_data_to_player_inventory, verify_character_in_town, verify_character_user,
    },
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

    let (inventory_data, _) =
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
    )
    .await?;

    let character_resources = db::characters::update_character_resources(
        &mut *tx,
        &payload.character_id,
        -item_bought.price,
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

    let (inventory_data, _) =
        db::characters_data::load_character_data(&mut *tx, &payload.character_id)
            .await?
            .ok_or(anyhow!("inventory not found"))?;

    let mut inventory =
        inventory_data_to_player_inventory(&master_store.items_store, inventory_data);

    let item_specs = (payload.item_index < inventory.bag.len())
        .then(|| inventory.bag.remove(payload.item_index))
        .ok_or(AppError::NotFound)?;

    let recipient_id = if let Some(username) = payload.recipient_name {
        let username = username.into_inner();
        Some(
            db::characters::get_character_by_name(&mut *tx, &username)
                .await?
                .ok_or(AppError::UserError(format!(
                    "character '{}' not found",
                    username
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

    let item_bought = db::market::buy_item(&mut tx, payload.item_index as i64, None)
        .await?
        .ok_or(AppError::NotFound)?;

    if item_bought.character_id != character.character_id {
        return Err(AppError::Forbidden);
    }

    db::market::sell_item(
        &mut tx,
        &payload.character_id,
        item_bought.recipient_id,
        payload.price,
        &items_controller::init_item_specs_from_store(
            &master_store.items_store,
            serde_json::from_value(item_bought.item_data).context("invalid data")?,
        )
        .ok_or(anyhow!("base item not found"))?,
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
