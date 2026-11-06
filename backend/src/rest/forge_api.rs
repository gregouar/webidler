use anyhow::Result;

use axum::{Extension, Json, Router, extract::State, middleware, routing::post};

use shared::{
    computations, constants,
    data::forge::affix_operation_price,
    http::{
        client::{
            ForgeAffixOperation, ForgeAffixRequest, ForgeImbueRequest, ForgeSacrificeRequest,
            ForgeTemperRequest, ForgeUpgradeRequest, GambleItemRequest,
        },
        server::{
            ForgeAffixResponse, ForgeImbueResponse, ForgeSacrificeResponse, ForgeTemperResponse,
            ForgeUpgradeResponse, GambleItemResponse,
        },
    },
};

use crate::{
    app_state::{AppState, MasterStore},
    auth::{self, User},
    db::{self},
    game::systems::{
        inventory_controller, items_controller,
        loot_generator::{self, GenerateLootTemplate},
        soul_crafting,
    },
    rest::utils::{buy_with_gems, load_player_town_inventory},
};

use super::AppError;

pub fn routes(app_state: AppState) -> Router<AppState> {
    Router::new()
        .route("/forge/affix", post(post_affix))
        .route("/forge/upgrade", post(post_upgrade))
        .route("/forge/temper", post(post_temper))
        .route("/forge/sacrifice", post(post_sacrifice))
        .route("/forge/imbue", post(post_imbue))
        .route("/forge/gamble", post(post_gamble))
        .layer(middleware::from_fn_with_state(
            app_state,
            auth::authorization_middleware,
        ))
}

pub async fn post_temper(
    State(db_pool): State<db::DbPool>,
    State(master_store): State<MasterStore>,
    Extension(user): Extension<User>,
    Json(payload): Json<ForgeTemperRequest>,
) -> Result<Json<ForgeTemperResponse>, AppError> {
    let mut tx = db_pool.begin().await?;

    let (_, mut inventory) =
        load_player_town_inventory(&mut tx, &master_store, &user, &payload.character_id).await?;

    let item = inventory
        .bag
        .get_mut(payload.item_index as usize)
        .ok_or(AppError::NotFound)?;

    let resource_gems = buy_with_gems(
        &mut tx,
        &payload.character_id,
        computations::temper_item_price(item).ok_or(AppError::UserError(
            "item cannot be tempered further".into(),
        ))?,
    )
    .await?;

    *item = items_controller::temper_item(&master_store.items_store, item)?;

    db::characters_data::save_character_inventory(&mut *tx, &payload.character_id, &inventory)
        .await?;

    tx.commit().await?;

    Ok(Json(ForgeTemperResponse {
        resource_gems,
        inventory,
    }))
}

pub async fn post_affix(
    State(db_pool): State<db::DbPool>,
    State(master_store): State<MasterStore>,
    Extension(user): Extension<User>,
    Json(payload): Json<ForgeAffixRequest>,
) -> Result<Json<ForgeAffixResponse>, AppError> {
    let mut tx = db_pool.begin().await?;

    let (_, mut inventory) =
        load_player_town_inventory(&mut tx, &master_store, &user, &payload.character_id).await?;

    let item = inventory
        .bag
        .get_mut(payload.item_index as usize)
        .ok_or(AppError::NotFound)?;

    let resource_gems = buy_with_gems(
        &mut tx,
        &payload.character_id,
        affix_operation_price(
            payload.operation,
            item.modifiers.count_presuffixes(),
            &item.base,
        )
        .ok_or(AppError::UserError("forge operation unavailable".into()))?,
    )
    .await?;

    if !match payload.operation {
        ForgeAffixOperation::Add(affix_type) => loot_generator::add_affix(
            &master_store.items_store,
            &item.base,
            &mut item.modifiers,
            affix_type,
        ),
        ForgeAffixOperation::Remove => {
            loot_generator::remove_affix(&master_store.items_store, &item.base, &mut item.modifiers)
        }
    } {
        return Err(AppError::UserError("forge operation failed".into()));
    }

    *item = items_controller::create_item_specs(
        &master_store.items_store,
        item.base.clone(),
        item.modifiers.clone(),
        0.0,
    );

    db::characters_data::save_character_inventory(&mut *tx, &payload.character_id, &inventory)
        .await?;

    tx.commit().await?;

    Ok(Json(ForgeAffixResponse {
        resource_gems,
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

    let (_, mut inventory) =
        load_player_town_inventory(&mut tx, &master_store, &user, &payload.character_id).await?;

    let item = inventory
        .bag
        .get_mut(payload.item_index as usize)
        .ok_or(AppError::NotFound)?;

    let resource_gems = buy_with_gems(
        &mut tx,
        &payload.character_id,
        computations::upgrade_item_price(item).ok_or(AppError::UserError(
            "maximum upgrade level reached for that item".into(),
        ))?,
    )
    .await?;

    *item = items_controller::upgrade_item(&master_store.items_store, item)?;

    db::characters_data::save_character_inventory(&mut *tx, &payload.character_id, &inventory)
        .await?;

    tx.commit().await?;

    Ok(Json(ForgeUpgradeResponse {
        resource_gems,
        inventory,
    }))
}

pub async fn post_sacrifice(
    State(db_pool): State<db::DbPool>,
    State(master_store): State<MasterStore>,
    Extension(user): Extension<User>,
    Json(payload): Json<ForgeSacrificeRequest>,
) -> Result<Json<ForgeSacrificeResponse>, AppError> {
    let mut tx = db_pool.begin().await?;

    let (_, mut inventory) =
        load_player_town_inventory(&mut tx, &master_store, &user, &payload.character_id).await?;

    let bag_index = payload.item_index as usize;
    let item = inventory.bag.get(bag_index).ok_or(AppError::NotFound)?;

    let resource_gems = buy_with_gems(
        &mut tx,
        &payload.character_id,
        computations::sacrifice_price(&item.affixes),
    )
    .await?;

    inventory.bag[bag_index] = soul_crafting::sacrifice(&master_store.items_store, item)?;
    db::characters_data::save_character_inventory(&mut *tx, &payload.character_id, &inventory)
        .await?;
    tx.commit().await?;

    Ok(Json(ForgeSacrificeResponse {
        resource_gems,
        inventory,
    }))
}

pub async fn post_imbue(
    State(db_pool): State<db::DbPool>,
    State(master_store): State<MasterStore>,
    Extension(user): Extension<User>,
    Json(payload): Json<ForgeImbueRequest>,
) -> Result<Json<ForgeImbueResponse>, AppError> {
    let mut tx = db_pool.begin().await?;

    let (_, mut inventory) =
        load_player_town_inventory(&mut tx, &master_store, &user, &payload.character_id).await?;

    let item_index = payload.item_index as usize;
    let soul_index = payload.soul_index as usize;

    let soul = inventory
        .bag
        .get(soul_index)
        .cloned()
        .ok_or(AppError::NotFound)?;
    let item = inventory
        .bag
        .get_mut(item_index)
        .ok_or(AppError::NotFound)?;
    *item = soul_crafting::imbue(&master_store.items_store, item, soul)?;

    inventory.bag.remove(soul_index);

    db::characters_data::save_character_inventory(&mut *tx, &payload.character_id, &inventory)
        .await?;
    tx.commit().await?;

    Ok(Json(ForgeImbueResponse { inventory }))
}

pub async fn post_gamble(
    State(db_pool): State<db::DbPool>,
    State(master_store): State<MasterStore>,
    Extension(user): Extension<User>,
    Json(payload): Json<GambleItemRequest>,
) -> Result<Json<GambleItemResponse>, AppError> {
    if !constants::GAMBLE_ITEM_CATEGORIES.contains(&payload.item_category) {
        return Err(AppError::UserError("forbidden item category".into()));
    }
    let mut tx = db_pool.begin().await?;

    let (character, mut inventory) =
        load_player_town_inventory(&mut tx, &master_store, &user, &payload.character_id).await?;

    let resource_gems = buy_with_gems(
        &mut tx,
        &payload.character_id,
        computations::gamble_price(character.max_area_level as u16),
    )
    .await?;

    match loot_generator::generate_loot(
        &master_store.gamble_table.loot_table,
        &master_store.items_store,
        &GenerateLootTemplate {
            allow_unique: true,
            max_base: payload.item_category.is_some(),
            max_affixes: true,
            filter_category: payload.item_category,
            prevent_categories: Default::default(),
        },
        character.max_area_level as u16,
        0,
        false,
        master_store.gamble_table.item_rarity,
        0.0,
    ) {
        Some(item_specs) => inventory_controller::store_item_to_bag(&mut inventory, item_specs)?,
        None => return Err(AppError::UserError("not item found".into())),
    }

    db::characters_data::save_character_inventory(&mut *tx, &payload.character_id, &inventory)
        .await?;

    tx.commit().await?;

    Ok(Json(GambleItemResponse {
        resource_gems,
        inventory,
    }))
}
