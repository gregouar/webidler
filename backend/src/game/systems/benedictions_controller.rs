use std::collections::HashMap;

use shared::data::{
    stat_effect::EffectsMap,
    temple::{BenedictionEffect, PlayerBenedictions},
    user::UserCharacterId,
};
use sqlx::Transaction;

use crate::{
    app_state::MasterStore,
    db::{self, pool::Database},
    game::data::master_store::BenedictionsStore,
    rest::AppError,
};

pub fn generate_effects_map_from_benedictions(
    benedictions_store: &BenedictionsStore,
    player_benedictions: &PlayerBenedictions,
) -> EffectsMap {
    player_benedictions
        .purchased_benedictions
        .iter()
        .filter_map(|(benediction_id, benediction_state)| {
            benedictions_store
                .get(benediction_id)
                .and_then(|benediction| {
                    benediction.compute_stat_effect(benediction_state.upgrade_level)
                })
        })
        .fold(EffectsMap(HashMap::new()), |mut effects_map, effect| {
            *effects_map
                .0
                .entry((effect.stat.clone(), effect.modifier))
                .or_default() += effect.value;
            effects_map
        })
}

pub fn find_benediction_value(
    benedictions_store: &BenedictionsStore,
    player_benedictions: &PlayerBenedictions,
    benediction_effect: BenedictionEffect,
) -> f64 {
    player_benedictions
        .purchased_benedictions
        .iter()
        .filter_map(|(benediction_id, benediction_state)| {
            benedictions_store
                .get(benediction_id)
                .and_then(|benediction| {
                    if benediction_effect == benediction.effect {
                        benediction.compute_value(benediction_state.upgrade_level)
                    } else {
                        None
                    }
                })
        })
        .fold(0.0, |total, value| total + value)
}

pub async fn update_benedictions(
    tx: &mut Transaction<'_, Database>,
    master_store: &MasterStore,
    character_id: &UserCharacterId,
    resource_gold: f64,
    player_benedictions: &PlayerBenedictions,
) -> Result<(), AppError> {
    let (_, _, prev_benedictions) =
        db::characters_data::load_character_data(&mut **tx, character_id)
            .await?
            .unwrap_or_default();

    validate_benedictions(&master_store.benedictions_store, player_benedictions)?;

    let cost = compute_benedictions_cost(&master_store.benedictions_store, player_benedictions)
        - compute_benedictions_cost(&master_store.benedictions_store, &prev_benedictions);

    if cost > resource_gold {
        return Err(AppError::UserError("not enough gold".to_string()));
    }

    db::characters::update_character_resources(&mut **tx, character_id, 0.0, 0.0, -cost).await?;

    db::characters_data::save_character_benedictions(&mut **tx, character_id, player_benedictions)
        .await?;

    Ok(())
}

pub fn compute_benedictions_cost(
    benedictions_store: &BenedictionsStore,
    player_benedictions: &PlayerBenedictions,
) -> f64 {
    player_benedictions
        .purchased_benedictions
        .iter()
        .map(|(benediction_id, benediction_state)| {
            benedictions_store
                .get(benediction_id)
                .map(|benediction_specs| {
                    benediction_specs.compute_total_price(benediction_state.upgrade_level)
                })
                .unwrap_or_default()
        })
        .sum()
}

pub fn validate_benedictions(
    benedictions_store: &BenedictionsStore,
    player_benedictions: &PlayerBenedictions,
) -> anyhow::Result<()> {
    for (benediction_id, benediction_state) in player_benedictions.purchased_benedictions.iter() {
        if benediction_state.upgrade_level
            > benedictions_store
                .get(benediction_id)
                .map(|benediction_specs| benediction_specs.max_upgrade_level.unwrap_or(u64::MAX))
                .unwrap_or_default()
        {
            return Err(anyhow::anyhow!("invalid benedictions"));
        }
    }
    Ok(())
}
