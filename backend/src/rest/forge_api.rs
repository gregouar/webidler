use anyhow::Result;

use axum::{extract::State, middleware, routing::post, Extension, Json, Router};

use shared::{
    data::{
        forge::{affix_price, PREFIX_PRICE_FACTOR, SUFFIX_PRICE_FACTOR},
        item_affix::AffixType,
        player::EquippedSlot,
    },
    http::{client::ForgeAddAffixRequest, server::ForgeAddAffixResponse},
};

use crate::{
    app_state::{AppState, MasterStore},
    auth::{self, CurrentUser},
    db::{self},
    game::{
        data::inventory_data::inventory_data_to_player_inventory,
        systems::loot_generator::add_affix,
    },
    rest::utils::{verify_character_in_town, verify_character_user},
};

use super::AppError;

pub fn routes(app_state: AppState) -> Router<AppState> {
    Router::new()
        .route("/forge/add_affix", post(post_add_affix))
        .layer(middleware::from_fn_with_state(
            app_state,
            auth::authorization_middleware,
        ))
}

pub async fn post_add_affix(
    State(db_pool): State<db::DbPool>,
    State(master_store): State<MasterStore>,
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<ForgeAddAffixRequest>,
) -> Result<Json<ForgeAddAffixResponse>, AppError> {
    let mut tx = db_pool.begin().await?;

    let character = db::characters::read_character(&mut *tx, &payload.character_id)
        .await?
        .ok_or(AppError::NotFound)?;

    verify_character_user(&character, &current_user)?;
    verify_character_in_town(&character)?;

    let (inventory_data, _) =
        db::characters_data::load_character_data(&mut *tx, &payload.character_id)
            .await?
            .ok_or(AppError::UserError("newbies can't forge items".into()))?;

    let mut inventory =
        inventory_data_to_player_inventory(&master_store.items_store, inventory_data);

    let item = if payload.item_index < 9 {
        inventory
            .equipped
            .get_mut(&(payload.item_index as usize).try_into()?)
            .and_then(|equipped_item| match equipped_item {
                EquippedSlot::MainSlot(item_specs) => Some(item_specs.as_mut()),
                _ => None,
            })
    } else {
        inventory
            .bag
            .get_mut(payload.item_index.saturating_sub(9) as usize)
    }
    .ok_or(AppError::NotFound)?;

    let price = affix_price(item.modifiers.count_nonunique_affixes())
        .ok_or(AppError::UserError("cannot add more affixes".into()))?
        * match payload.affix_type {
            Some(AffixType::Prefix) => PREFIX_PRICE_FACTOR,
            Some(AffixType::Suffix) => SUFFIX_PRICE_FACTOR,
            _ => 1.0,
        };

    let character_resources =
        db::characters::update_character_resources(&mut *tx, &payload.character_id, -price, 0.0)
            .await?;

    if character_resources.resource_gems < 0.0 {
        return Err(AppError::UserError("not enough gems".into()));
    }

    // Decrease item level to match with player
    item.modifiers.level = item.modifiers.level.min(character.max_area_level as u16);

    if !add_affix(
        &item.base,
        &mut item.modifiers,
        payload.affix_type,
        &master_store.item_affixes_table,
        &master_store.item_adjectives_table,
        &master_store.item_nouns_table,
    ) {
        return Err(AppError::UserError("failed to add affix".into()));
    }

    db::characters_data::save_character_inventory(&mut *tx, &payload.character_id, &inventory)
        .await?;

    tx.commit().await?;

    Ok(Json(ForgeAddAffixResponse {
        resource_gems: character_resources.resource_gems,
        inventory,
    }))
}
