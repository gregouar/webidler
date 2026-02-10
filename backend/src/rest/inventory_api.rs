use anyhow::Result;

use axum::{Extension, Json, Router, extract::State, middleware, routing::post};

use shared::{
    data::area::AreaLevel,
    http::{
        client::{InventoryDeleteRequest, InventoryEquipRequest, InventoryUnequipRequest},
        server::{InventoryDeleteResponse, InventoryEquipResponse, InventoryUnequipResponse},
    },
};

use crate::{
    app_state::{AppState, MasterStore},
    auth::{self, CurrentUser},
    db,
    game::{
        data::inventory_data::inventory_data_to_player_inventory, systems::inventory_controller,
    },
    rest::utils::{verify_character_in_town, verify_character_user},
};

use super::AppError;

pub fn routes(app_state: AppState) -> Router<AppState> {
    Router::new()
        .route("/inventory/equip", post(post_equip_item))
        .route("/inventory/unequip", post(post_unequip_item))
        .route("/inventory/delete", post(post_delete_items))
        .layer(middleware::from_fn_with_state(
            app_state,
            auth::authorization_middleware,
        ))
}

pub async fn post_equip_item(
    State(db_pool): State<db::DbPool>,
    State(master_store): State<MasterStore>,
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<InventoryEquipRequest>,
) -> Result<Json<InventoryEquipResponse>, AppError> {
    let mut tx = db_pool.begin().await?;

    let character = db::characters::read_character(&mut *tx, &payload.character_id)
        .await?
        .ok_or(AppError::NotFound)?;

    verify_character_user(&character, &current_user)?;
    verify_character_in_town(&character)?;

    let (inventory_data, _, _) =
        db::characters_data::load_character_data(&mut *tx, &payload.character_id)
            .await?
            .ok_or(AppError::UserError("newbies don't have inventory".into()))?;

    let mut inventory =
        inventory_data_to_player_inventory(&master_store.items_store, inventory_data);

    inventory_controller::equip_item_from_bag(
        character.max_area_level as AreaLevel,
        &mut inventory,
        payload.item_index,
    )?;

    db::characters_data::save_character_inventory(&mut *tx, &payload.character_id, &inventory)
        .await?;

    tx.commit().await?;

    Ok(Json(InventoryEquipResponse { inventory }))
}

pub async fn post_unequip_item(
    State(db_pool): State<db::DbPool>,
    State(master_store): State<MasterStore>,
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<InventoryUnequipRequest>,
) -> Result<Json<InventoryUnequipResponse>, AppError> {
    let mut tx = db_pool.begin().await?;

    let character = db::characters::read_character(&mut *tx, &payload.character_id)
        .await?
        .ok_or(AppError::NotFound)?;

    verify_character_user(&character, &current_user)?;
    verify_character_in_town(&character)?;

    let (inventory_data, _, _) =
        db::characters_data::load_character_data(&mut *tx, &payload.character_id)
            .await?
            .ok_or(AppError::UserError("newbies don't have inventory".into()))?;

    let mut inventory =
        inventory_data_to_player_inventory(&master_store.items_store, inventory_data);

    inventory_controller::unequip_item_to_bag(&mut inventory, payload.item_slot)?;

    db::characters_data::save_character_inventory(&mut *tx, &payload.character_id, &inventory)
        .await?;

    tx.commit().await?;

    Ok(Json(InventoryUnequipResponse { inventory }))
}

pub async fn post_delete_items(
    State(db_pool): State<db::DbPool>,
    State(master_store): State<MasterStore>,
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<InventoryDeleteRequest>,
) -> Result<Json<InventoryDeleteResponse>, AppError> {
    let mut tx = db_pool.begin().await?;

    let character = db::characters::read_character(&mut *tx, &payload.character_id)
        .await?
        .ok_or(AppError::NotFound)?;

    verify_character_user(&character, &current_user)?;
    verify_character_in_town(&character)?;

    let (inventory_data, _, _) =
        db::characters_data::load_character_data(&mut *tx, &payload.character_id)
            .await?
            .ok_or(AppError::UserError("newbies don't have inventory".into()))?;

    let mut inventory =
        inventory_data_to_player_inventory(&master_store.items_store, inventory_data);

    let mut item_indexes = payload.item_indexes;
    item_indexes.sort_by_key(|&i| i);
    for &item_index in item_indexes.iter().rev() {
        inventory_controller::remove_item_from_bag(&mut inventory, item_index)?;
    }

    db::characters_data::save_character_inventory(&mut *tx, &payload.character_id, &inventory)
        .await?;

    tx.commit().await?;

    Ok(Json(InventoryDeleteResponse { inventory }))
}
