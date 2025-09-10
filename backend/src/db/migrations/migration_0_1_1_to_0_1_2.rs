use crate::db::pool::{DbExecutor, DbPool};

pub async fn migrate(db_pool: &DbPool) -> anyhow::Result<()> {
    let mut tx = db_pool.begin().await?;

    stop_all_grinds(&mut *tx).await?;

    tx.commit().await?;
    Ok(())
}

async fn stop_all_grinds<'c>(executor: impl DbExecutor<'c>) -> anyhow::Result<()> {
    sqlx::query!("DELETE FROM saved_game_instances WHERE data_version = '0.1.1'")
        .execute(executor)
        .await?;
    Ok(())
}
