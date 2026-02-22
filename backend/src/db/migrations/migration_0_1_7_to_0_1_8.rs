use sqlx::Transaction;

use crate::db::pool::{Database, DbExecutor, DbPool};

pub async fn migrate(db_pool: &DbPool) -> anyhow::Result<()> {
    let mut tx = db_pool.begin().await?;

    stop_all_grinds(&mut *tx).await?;

    migrate_leaderboard(&mut tx).await?;

    tx.commit().await?;
    Ok(())
}

async fn stop_all_grinds<'c>(executor: impl DbExecutor<'c>) -> anyhow::Result<()> {
    sqlx::query!("DELETE FROM saved_game_instances WHERE data_version <= '0.1.7'")
        .execute(executor)
        .await?;
    Ok(())
}

async fn migrate_leaderboard<'c>(
    executor: &mut Transaction<'static, Database>,
) -> anyhow::Result<()> {
    sqlx::query!(
        "UPDATE game_stats
        SET
            area_level = CASE
                WHEN area_id = 'forest.json' THEN area_level - 10
                WHEN area_id = 'witch_lair.json' THEN area_level - 20
                WHEN area_id = 'castle.json' THEN area_level - 30
                WHEN area_id = 'island.json' THEN area_level - 40
                WHEN area_id = 'peaks.json' THEN area_level - 50
                ELSE area_level
            END,
            data_version = '0.1.8_' || data_version
        WHERE data_version <= '0.1.7'"
    )
    .execute(&mut **executor)
    .await?;

    sqlx::query!(
        "DELETE FROM game_stats
        WHERE area_level <= 0"
    )
    .execute(&mut **executor)
    .await?;

    Ok(())
}
