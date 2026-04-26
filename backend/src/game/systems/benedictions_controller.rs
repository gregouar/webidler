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
    EffectsMap(
        player_benedictions
            .categories
            .iter()
            .flat_map(|(category_id, player_category)| {
                let store_category = benedictions_store.get(category_id);

                player_category.purchased_benedictions.iter().filter_map(
                    move |(benediction_id, upgrade_level)| {
                        let specs = store_category?.benedictions.get(benediction_id)?;

                        specs.compute_stat_effect(*upgrade_level)
                    },
                )
            })
            .fold(HashMap::new(), |mut effects, stat_effect| {
                *effects
                    .entry((
                        stat_effect.stat.clone(),
                        stat_effect.modifier,
                        stat_effect.bypass_ignore,
                    ))
                    .or_default() += stat_effect.value;

                effects
            }),
    )
}

pub fn find_benediction_value(
    benedictions_store: &BenedictionsStore,
    player_benedictions: &PlayerBenedictions,
    benediction_effect: &BenedictionEffect,
) -> f64 {
    player_benedictions
        .categories
        .iter()
        .flat_map(|(category_id, player_category)| {
            let store_category = benedictions_store.get(category_id);

            player_category.purchased_benedictions.iter().filter_map(
                move |(benediction_id, upgrade_level)| {
                    let benediction = store_category?.benedictions.get(benediction_id)?;
                    if *benediction_effect == benediction.effect {
                        benediction.compute_value(*upgrade_level)
                    } else {
                        None
                    }
                },
            )
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

    validate_benedictions(
        &master_store.benedictions_store,
        &prev_benedictions,
        player_benedictions,
    )?;

    let cost = compute_benedictions_cost(&master_store.benedictions_store, player_benedictions)
        - compute_benedictions_cost(&master_store.benedictions_store, &prev_benedictions);

    if cost > resource_gold {
        return Err(AppError::UserError("not enough gold".to_string()));
    }

    db::characters::update_character_resources(&mut **tx, character_id, 0.0, 0.0, -cost, 0.0)
        .await?;

    db::characters_data::save_character_benedictions(&mut **tx, character_id, player_benedictions)
        .await?;

    Ok(())
}

pub fn compute_benedictions_cost(
    benedictions_store: &BenedictionsStore,
    player_benedictions: &PlayerBenedictions,
) -> f64 {
    player_benedictions
        .categories
        .iter()
        .map(|(category_id, player_category)| {
            benedictions_store
                .get(category_id)
                .map(|category_specs| {
                    category_specs.compute_total_price(player_category.upgrade_level)
                })
                .unwrap_or_default()
        })
        .sum()
}

pub fn validate_benedictions(
    benedictions_store: &BenedictionsStore,
    prev_benedictions: &PlayerBenedictions,
    player_benedictions: &PlayerBenedictions,
) -> anyhow::Result<()> {
    for (category_id, player_category) in player_benedictions.categories.iter() {
        let category_specs = benedictions_store
            .get(category_id)
            .ok_or_else(|| anyhow::anyhow!("invalid benedictions"))?;

        if player_category.upgrade_level > category_specs.max_upgrade_level.unwrap_or(u64::MAX) {
            return Err(anyhow::anyhow!("invalid benedictions"));
        }

        for benediction_id in player_category.purchased_benedictions.keys() {
            if !category_specs.benedictions.contains_key(benediction_id) {
                return Err(anyhow::anyhow!("invalid benedictions"));
            }
        }

        let total_level: u64 = player_category
            .purchased_benedictions
            .values()
            .copied()
            .sum();
        if total_level > player_category.upgrade_level {
            return Err(anyhow::anyhow!("invalid benedictions"));
        }
    }

    for (category_id, prev_category) in prev_benedictions.categories.iter() {
        let upgrade_level = player_benedictions
            .categories
            .get(category_id)
            .map(|category| category.upgrade_level)
            .unwrap_or_default();

        if upgrade_level < prev_category.upgrade_level {
            return Err(anyhow::anyhow!("invalid benedictions"));
        }
    }

    Ok(())
}
