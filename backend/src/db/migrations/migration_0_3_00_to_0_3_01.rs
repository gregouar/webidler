use anyhow::Context;
use sqlx::{Transaction, types::JsonValue};

use shared::data::{item::ItemModifiers, item_affix::AffixType, user::UserCharacterId};

use crate::{
    db::{
        characters_data::{
            CharacterDataEntry, upsert_character_inventory_data, upsert_character_passives_data,
        },
        pool::{Database, DbExecutor, DbPool},
    },
    game::data::{inventory_data::InventoryData, passives::PassivesTreeAscensionData},
};

const TARGET_DATA_VERSION: &str = "0.3.01";

pub async fn migrate(db_pool: &DbPool) -> anyhow::Result<()> {
    let mut tx = db_pool.begin().await?;

    stop_all_grinds(&mut *tx).await?;

    migrate_character_data(&mut tx)
        .await
        .context("migrate_character_data")?;
    migrate_stash_items(&mut tx)
        .await
        .context("migrate_stash_items")?;

    tx.commit().await?;
    Ok(())
}

async fn stop_all_grinds<'c>(executor: impl DbExecutor<'c>) -> anyhow::Result<()> {
    sqlx::query!(
        "DELETE FROM saved_game_instances WHERE data_version < $1",
        TARGET_DATA_VERSION
    )
    .execute(executor)
    .await?;
    Ok(())
}
async fn migrate_character_data(
    executor: &mut Transaction<'static, Database>,
) -> anyhow::Result<()> {
    let characters_data = sqlx::query_as!(
        CharacterDataEntry,
        r#"
        SELECT
            character_id as "character_id: UserCharacterId",
            data_version,
            inventory_data,
            passives_data,
            benedictions_data,
            skill_masteries_data,
            pets_data,
            created_at,
            updated_at
         FROM characters_data
         WHERE data_version < $1
         "#,
        TARGET_DATA_VERSION
    )
    .fetch_all(&mut **executor)
    .await?;

    for character_data in characters_data {
        let mut inventory_data: InventoryData =
            rmp_serde::from_slice(&character_data.inventory_data)
                .context(format!("inventory of '{}'", character_data.character_id))?;

        for item_modifiers in inventory_data.equipped.values_mut() {
            migrate_item(item_modifiers)
        }
        for item_modifiers in inventory_data.bag.iter_mut() {
            migrate_item(item_modifiers)
        }

        upsert_character_inventory_data(
            &mut **executor,
            &character_data.character_id,
            rmp_serde::to_vec(&inventory_data)?,
        )
        .await?;

        if let Some(passives_data) = character_data.passives_data {
            let mut passives_data: PassivesTreeAscensionData =
                rmp_serde::from_slice(&passives_data)
                    .context(format!("passives of '{}'", character_data.character_id))?;

            for item_modifiers in passives_data.socketed_nodes.values_mut() {
                migrate_item(item_modifiers)
            }

            upsert_character_passives_data(
                &mut **executor,
                &character_data.character_id,
                rmp_serde::to_vec(&passives_data)?,
            )
            .await?;
        }

        sqlx::query!(
            "UPDATE characters_data SET data_version = $1 WHERE character_id = $2",
            TARGET_DATA_VERSION,
            character_data.character_id,
        )
        .execute(&mut **executor)
        .await?;
    }

    Ok(())
}

async fn migrate_stash_items(executor: &mut Transaction<'static, Database>) -> anyhow::Result<()> {
    let market_entries = sqlx::query!(
        r#"
        SELECT
            stash_item_id,
            item_data as "item_data: JsonValue"
        FROM stash_items
        WHERE data_version < $1
        "#,
        TARGET_DATA_VERSION
    )
    .fetch_all(&mut **executor)
    .await?;

    let new_market_entries = market_entries
        .into_iter()
        .map(|market_entry| {
            let mut item_modifiers: ItemModifiers = serde_json::from_value(market_entry.item_data)?;
            migrate_item(&mut item_modifiers);
            Ok((
                market_entry.stash_item_id,
                serde_json::to_value(&item_modifiers)?,
            ))
        })
        .collect::<anyhow::Result<Vec<_>>>()?;

    for (stash_item_id, item_data) in new_market_entries {
        sqlx::query!(
            "UPDATE stash_items SET item_data = $1, data_version = $2 WHERE stash_item_id = $3",
            item_data,
            TARGET_DATA_VERSION,
            stash_item_id,
        )
        .execute(&mut **executor)
        .await?;
    }
    Ok(())
}

fn migrate_item(item_modifiers: &mut ItemModifiers) {
    if let Some(affix) = item_modifiers
        .affixes
        .iter_mut()
        .filter(|affix| matches!(affix.affix_type, AffixType::Prefix | AffixType::Suffix))
        .nth(4)
    {
        affix.affix_type = AffixType::Soul
    }
}
