use anyhow::{anyhow, Result};

use axum::{extract::State, middleware, routing::post, Extension, Json, Router};

use shared::{
    data::market::MarketItem,
    http::{
        client::{BrowseMarketItemsRequest, BuyMarketItemRequest, SellMarketItemRequest},
        server::{BrowseMarketItemsResponse, BuyMarketItemResponse, SellMarketItemResponse},
    },
};

use crate::{
    app_state::{AppState, MasterStore},
    auth::{self, CurrentUser},
    db,
    game::systems::items_controller,
    rest::utils::{
        inventory_data_to_player_inventory, verify_character_in_town, verify_character_user,
    },
};

use super::AppError;

pub fn routes(app_state: AppState) -> Router<AppState> {
    let auth_routes = Router::new()
        .route("/market/buy", post(post_buy_market_item))
        .route("/market/sell", post(post_sell_market_item))
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
    let (items, has_more) = db::market::load_market_items(
        &db_pool,
        &payload.character_id,
        payload.own_listings,
        payload.skip as i64,
        payload.limit.into_inner(),
    )
    .await?;

    Ok(Json(BrowseMarketItemsResponse {
        items: items
            .into_iter()
            .filter_map(|market_item_entry| {
                Some(MarketItem {
                    item_id: market_item_entry.item_id,
                    seller: market_item_entry.character_id,
                    private_sale: market_item_entry.private_sale,
                    price: market_item_entry.price,

                    item_specs: items_controller::init_item_specs_from_store(
                        &master_store.items_store,
                        market_item_entry.item_modifiers,
                    )?,

                    created_at: market_item_entry.created_at.into(),
                })
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

    let item_bought = db::market::buy_item(&mut tx, payload.item_index as i64)
        .await?
        .ok_or(AppError::NotFound)?;

    if let Some(private_sale) = item_bought.private_sale {
        if private_sale != character.character_id {
            return Err(AppError::Forbidden);
        }
    }

    if character.max_area_level < item_bought.item_level as i32 {
        return Err(AppError::UserError("character level too low".to_string()));
    }

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

    // TODO: VERIFY CHARACTER LEVEL

    db::characters::update_character_resources(
        &mut *tx,
        &item_bought.character_id,
        item_bought.price,
        0.0,
    )
    .await?;

    inventory.bag.push(
        items_controller::init_item_specs_from_store(
            &master_store.items_store,
            item_bought.item_modifiers,
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

    // TODO: MAX ITEMS ON SALE

    let (inventory_data, _) =
        db::characters_data::load_character_data(&mut *tx, &payload.character_id)
            .await?
            .ok_or(anyhow!("inventory not found"))?;

    let mut inventory =
        inventory_data_to_player_inventory(&master_store.items_store, inventory_data);

    let item_specs = (payload.item_index < inventory.bag.len())
        .then(|| inventory.bag.remove(payload.item_index))
        .ok_or(AppError::NotFound)?;

    let private_sale = if let Some(username) = payload.private_offer {
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

    db::market::sell_item(
        &mut tx,
        &payload.character_id,
        private_sale,
        payload.price,
        &item_specs,
    )
    .await?;

    db::characters_data::save_character_inventory(&mut *tx, &payload.character_id, &inventory)
        .await?;

    tx.commit().await?;

    Ok(Json(SellMarketItemResponse { inventory }))
}
