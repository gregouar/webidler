use anyhow::Result;

use axum::{Extension, Json, Router, extract::State, middleware, routing::post};

use shared::{
    computations, constants,
    data::forge::affix_operation_price,
    http::{
        client::{ForgeAffixOperation, ForgeAffixRequest, ForgeUpgradeRequest, GambleItemRequest},
        server::{ForgeAffixResponse, ForgeUpgradeResponse, GambleItemResponse},
    },
};

use crate::{
    app_state::{AppState, MasterStore},
    auth::{self, User},
    db::{self},
    game::{
        data::inventory_data::inventory_data_to_player_inventory,
        systems::{inventory_controller, items_controller, loot_generator},
    },
    rest::utils::{verify_character_in_town, verify_character_user},
};

use super::AppError;

pub fn routes(app_state: AppState) -> Router<AppState> {
    Router::new()
        .route("/forge/affix", post(post_affix))
        .route("/forge/upgrade", post(post_upgrade))
        .route("/forge/gamble", post(post_gamble))
        .layer(middleware::from_fn_with_state(
            app_state,
            auth::authorization_middleware,
        ))
}

pub async fn post_affix(
    State(db_pool): State<db::DbPool>,
    State(master_store): State<MasterStore>,
    Extension(user): Extension<User>,
    Json(payload): Json<ForgeAffixRequest>,
) -> Result<Json<ForgeAffixResponse>, AppError> {
    let mut tx = db_pool.begin().await?;

    let character = db::characters::read_character(&mut *tx, &payload.character_id)
        .await?
        .ok_or(AppError::NotFound)?;

    verify_character_user(&character, &user)?;
    verify_character_in_town(&character)?;

    let (inventory_data, _, _) =
        db::characters_data::load_character_data(&mut *tx, &payload.character_id)
            .await?
            .ok_or(AppError::UserError("newbies can't forge items".into()))?;

    let mut inventory =
        inventory_data_to_player_inventory(&master_store.items_store, inventory_data);

    let item = inventory
        .nth_mut(payload.item_index as usize)
        .ok_or(AppError::NotFound)?;

    let price = affix_operation_price(payload.operation, item.modifiers.count_nonunique_affixes())
        .ok_or(AppError::UserError("forge operation unavailable".into()))?;

    let character_resources = db::characters::update_character_resources(
        &mut *tx,
        &payload.character_id,
        -price,
        0.0,
        0.0,
        0.0,
    )
    .await?;

    if character_resources.resource_gems < 0.0 {
        return Err(AppError::UserError("not enough gems".into()));
    }

    if !match payload.operation {
        ForgeAffixOperation::Add(affix_type) => loot_generator::add_affix(
            &item.base,
            &mut item.modifiers,
            affix_type,
            &master_store.item_affixes_table,
            &master_store.item_adjectives_table,
            &master_store.item_nouns_table,
        ),
        ForgeAffixOperation::Remove => loot_generator::remove_affix(
            &item.base,
            &mut item.modifiers,
            &master_store.item_adjectives_table,
            &master_store.item_nouns_table,
        ),
    } {
        return Err(AppError::UserError("forge operation failed".into()));
    }

    db::characters_data::save_character_inventory(&mut *tx, &payload.character_id, &inventory)
        .await?;

    tx.commit().await?;

    Ok(Json(ForgeAffixResponse {
        resource_gems: character_resources.resource_gems,
        inventory,
    }))
}

pub async fn post_upgrade(
    State(db_pool): State<db::DbPool>,
    State(master_store): State<MasterStore>,
    Extension(user): Extension<User>,
    Json(payload): Json<ForgeUpgradeRequest>,
) -> Result<Json<ForgeUpgradeResponse>, AppError> {
    let mut tx = db_pool.begin().await?;

    let character = db::characters::read_character(&mut *tx, &payload.character_id)
        .await?
        .ok_or(AppError::NotFound)?;

    verify_character_user(&character, &user)?;
    verify_character_in_town(&character)?;

    let (inventory_data, _, _) =
        db::characters_data::load_character_data(&mut *tx, &payload.character_id)
            .await?
            .ok_or(AppError::UserError("newbies can't forge items".into()))?;

    let mut inventory =
        inventory_data_to_player_inventory(&master_store.items_store, inventory_data);

    let item = inventory
        .nth_mut(payload.item_index as usize)
        .ok_or(AppError::NotFound)?;

    let price = computations::upgrade_item_price(item).ok_or(AppError::UserError(
        "maximum upgrade level reached for that item".into(),
    ))?;

    let character_resources = db::characters::update_character_resources(
        &mut *tx,
        &payload.character_id,
        -price,
        0.0,
        0.0,
        0.0,
    )
    .await?;

    if character_resources.resource_gems < 0.0 {
        return Err(AppError::UserError("not enough gems".into()));
    }

    *item = items_controller::upgrade_item(item)?;

    db::characters_data::save_character_inventory(&mut *tx, &payload.character_id, &inventory)
        .await?;

    tx.commit().await?;

    Ok(Json(ForgeUpgradeResponse {
        resource_gems: character_resources.resource_gems,
        inventory,
    }))
}

pub async fn post_gamble(
    State(db_pool): State<db::DbPool>,
    State(master_store): State<MasterStore>,
    Extension(user): Extension<User>,
    Json(payload): Json<GambleItemRequest>,
) -> Result<Json<GambleItemResponse>, AppError> {
    let mut tx = db_pool.begin().await?;

    let character = db::characters::read_character(&mut *tx, &payload.character_id)
        .await?
        .ok_or(AppError::NotFound)?;

    verify_character_user(&character, &user)?;
    verify_character_in_town(&character)?;

    if !constants::GAMBLE_ITEM_CATEGORIES.contains(&payload.item_category) {
        return Err(AppError::UserError("forbidden item category".into()));
    }

    let (inventory_data, _, _) =
        db::characters_data::load_character_data(&mut *tx, &payload.character_id)
            .await?
            .ok_or(AppError::UserError("newbies can't forge items".into()))?;

    let mut inventory =
        inventory_data_to_player_inventory(&master_store.items_store, inventory_data);

    let price = computations::gamble_price(character.max_area_level as u16);

    let character_resources = db::characters::update_character_resources(
        &mut *tx,
        &payload.character_id,
        -price,
        0.0,
        0.0,
        0.0,
    )
    .await?;

    if character_resources.resource_gems < 0.0 {
        return Err(AppError::UserError("not enough gems".into()));
    }

    // TODO: Only max base item & max amount of affixes
    match loot_generator::generate_loot(
        &master_store.gamble_table.loot_table,
        &master_store.items_store,
        &master_store.item_affixes_table,
        &master_store.item_adjectives_table,
        &master_store.item_nouns_table,
        character.max_area_level as u16,
        false,
        true,
        payload.item_category.is_some(),
        true,
        payload.item_category,
        master_store.gamble_table.item_rarity,
    ) {
        Some(item_specs) => inventory_controller::store_item_to_bag(&mut inventory, item_specs)?,
        None => return Err(AppError::UserError("not item found".into())),
    }

    db::characters_data::save_character_inventory(&mut *tx, &payload.character_id, &inventory)
        .await?;

    tx.commit().await?;

    Ok(Json(GambleItemResponse {
        resource_gems: character_resources.resource_gems,
        inventory,
    }))
}
