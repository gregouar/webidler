use sqlx::Transaction;

use crate::{
    app_state::MasterStore,
    db::{
        self,
        pool::{Database, DbPool},
    },
    game::systems::items_controller,
};

pub async fn migrate(db_pool: &DbPool, master_store: &MasterStore) -> anyhow::Result<()> {
    let mut tx = db_pool.begin().await?;

    migrate_market_items(&mut tx, master_store).await?;

    tx.commit().await?;
    Ok(())
}

async fn migrate_market_items(
    executor: &mut Transaction<'static, Database>,
    master_store: &MasterStore,
) -> anyhow::Result<()> {
    // let records = sqlx::query!(
    //     r#"
    //     SELECT
    //         market_id,
    //         created_at,
    //         updated_at
    //     FROM market
    //     WHERE data_version <= '0.1.4' AND deleted_at is NULL
    //     "#
    // )
    // .fetch_all(&mut **executor)
    // .await?;

    // for record in records {
    //     if let Some(market_entry) = db::market::buy_item(executor, record.market_id, None).await? {
    //         let market_id = db::market::sell_item(
    //             executor,
    //             &market_entry.character_id,
    //             market_entry.recipient_id,
    //             market_entry.price,
    //             &items_controller::init_item_specs_from_store(
    //                 &master_store.items_store,
    //                 serde_json::from_value(market_entry.item_data)?,
    //             )
    //             .ok_or(anyhow::anyhow!("base item not found"))?,
    //         )
    //         .await?;

    //         sqlx::query!(
    //             r#"
    //             UPDATE market SET
    //                 created_at = $1,
    //                 updated_at = $2
    //             WHERE market_id = $3
    //             "#,
    //             record.created_at,
    //             record.updated_at,
    //             market_id
    //         )
    //         .execute(&mut **executor)
    //         .await?;
    //     }
    // }

    Ok(())
}
