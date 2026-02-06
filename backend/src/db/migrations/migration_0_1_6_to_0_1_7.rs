use shared::data::{passive::PassivesTreeAscension, user::UserCharacterId};

use crate::{
    app_state::MasterStore,
    db::pool::{Database, DbExecutor, DbPool},
    game::systems::passives_controller,
};

pub async fn migrate(db_pool: &DbPool, master_store: &MasterStore) -> anyhow::Result<()> {
    let mut tx = db_pool.begin().await?;

    stop_all_grinds(&mut *tx).await?;

    // migrate_market_items(&mut tx).await?;
    // migrate_character_items(&mut tx).await?;

    tx.commit().await?;
    Ok(())
}

async fn stop_all_grinds<'c>(executor: impl DbExecutor<'c>) -> anyhow::Result<()> {
    sqlx::query!("DELETE FROM saved_game_instances WHERE data_version = '0.1.2'")
        .execute(executor)
        .await?;
    Ok(())
}

// async fn migrate_market_items(executor: &mut Transaction<'static, Database>) -> anyhow::Result<()> {
//     let market_entries = sqlx::query!(
//         r#"
//         SELECT
//             market_id,
//             item_data as "item_data: JsonValue"
//         FROM market_old
//         WHERE data_version IS NULL
//         "#
//     )
//     .fetch_all(&mut **executor)
//     .await?;

//     let new_market_entries = market_entries
//         .into_iter()
//         .map(|market_entry| {
//             let old_item_modifiers: OldItemModifiers =
//                 serde_json::from_value(market_entry.item_data)?;
//             let item_modifiers: ItemModifiers = old_item_modifiers.into();
//             Ok((
//                 market_entry.market_id,
//                 serde_json::to_value(&item_modifiers)?,
//             ))
//         })
//         .collect::<anyhow::Result<Vec<_>>>()?;

//     for (market_id, item_data) in new_market_entries {
//         sqlx::query!(
//             "UPDATE market_old SET item_data = $1, data_version = $2 WHERE market_id = $3",
//             item_data,
//             DATA_VERSION,
//             market_id,
//         )
//         .execute(&mut **executor)
//         .await?;
//     }

//     Ok(())
// }

// async fn migrate_character_items(
//     executor: &mut Transaction<'static, Database>,
// ) -> anyhow::Result<()> {
//     let characters_data = sqlx::query_as!(
//         CharacterDataEntry,
//         r#"
//         SELECT
//             character_id as "character_id: UserCharacterId",
//             data_version,
//             inventory_data,
//             passives_data,
//             benedictions_data,
//             created_at,
//             updated_at
//          FROM characters_data
//          WHERE data_version = '0.1.2'
//          "#,
//     )
//     .fetch_all(&mut **executor)
//     .await?;

//     for character_data in characters_data {
//         let old_inventory: OldInventoryData =
//             rmp_serde::from_slice(&character_data.inventory_data)?;
//         let inventory_data: InventoryData = old_inventory.into();
//         upsert_character_inventory_data(
//             &mut **executor,
//             &character_data.character_id,
//             rmp_serde::to_vec(&inventory_data)?,
//         )
//         .await?;
//     }

//     Ok(())
// }
