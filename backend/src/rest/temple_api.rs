use anyhow::{anyhow, Context, Result};

use axum::{extract::State, middleware, routing::post, Extension, Json, Router};

use shared::{
    constants::{MAX_MARKET_PRIVATE_LISTINGS, MAX_MARKET_PUBLIC_LISTINGS},
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
        systems::items_controller,
    },
    rest::utils::{verify_character_in_town, verify_character_user},
};

use super::AppError;

pub fn routes(app_state: AppState) -> Router<AppState> {
    let auth_routes: Router<AppState> = Router::new()
        // .route("/temple/buy", post(post_buy_benediction))
        .layer(middleware::from_fn_with_state(
            app_state,
            auth::authorization_middleware,
        ));

    Router::new()
        // .route("/market", post(post_browse_market))
        .merge(auth_routes)
}

// pub async fn post_browse_market(
//     State(db_pool): State<db::DbPool>,
//     State(master_store): State<MasterStore>,
//     Json(payload): Json<BrowseMarketItemsRequest>,
// ) -> Result<Json<BrowseMarketItemsResponse>, AppError> {
//     let (items, has_more) = db::market::read_market_items(
//         &db_pool,
//         &payload.character_id,
//         payload.own_listings,
//         payload.is_deleted,
//         payload.filters,
//         payload.skip as i64,
//         payload.limit.into_inner(),
//     )
//     .await?;

//     Ok(Json(BrowseMarketItemsResponse {
//         items: items
//             .into_iter()
//             .filter_map(|market_item_entry| {
//                 into_market_item(&master_store.items_store, market_item_entry)
//             })
//             .collect(),
//         has_more,
//     }))
// }

// pub async fn post_buy_benediction(
//     State(db_pool): State<db::DbPool>,
//     Extension(current_user): Extension<CurrentUser>,
//     Json(payload): Json<TempleBuyRequest>,
// ) -> Result<Json<TempleBuyResponse>, AppError> {
// }
