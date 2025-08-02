use anyhow;

use shared::messages::UserId;
use sqlx::FromRow;

use crate::{
    db::utc_datetime::UtcDateTime,
    game::{data::master_store, game_data::GameInstanceData},
};

use super::pool::DbPool;

#[derive(Debug, FromRow)]
pub struct SavedGameInstance {
    pub user_id: UserId,
    pub saved_at: UtcDateTime,
    pub game_data: Vec<u8>, // Assuming game_data is stored as a binary blob
}

pub async fn save_game_instance_data(
    pool: &DbPool,
    user_id: &UserId,
    game_instance_data: GameInstanceData,
) -> anyhow::Result<()> {
    Ok(upsert_saved_game_instance(pool, user_id, game_instance_data.to_bytes()?).await?)
}

async fn upsert_saved_game_instance(
    pool: &DbPool,
    user_id: &UserId,
    game_data: Vec<u8>,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO saved_game_instances (user_id, game_data) VALUES ($1, $2)
         ON CONFLICT(user_id) DO UPDATE SET game_data = EXCLUDED.game_data, saved_at = CURRENT_TIMESTAMP",
        user_id,
        game_data
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn load_game_instance_data(
    pool: &DbPool,
    master_store: &master_store::MasterStore,
    user_id: &UserId,
) -> anyhow::Result<Option<GameInstanceData>> {
    let saved_game_instance = load_saved_game_instance(pool, user_id).await?;
    if let Some(instance) = saved_game_instance {
        Ok(Some(GameInstanceData::from_bytes(
            master_store,
            &instance.game_data,
        )?))
    } else {
        Ok(None)
    }
}

async fn load_saved_game_instance(
    pool: &DbPool,
    user_id: &UserId,
) -> Result<Option<SavedGameInstance>, sqlx::Error> {
    let instance = sqlx::query_as!(
        SavedGameInstance,
        "SELECT user_id, saved_at, game_data FROM saved_game_instances WHERE user_id = $1",
        user_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(instance)
}

pub async fn delete_game_instance_data(pool: &DbPool, user_id: &UserId) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "DELETE FROM saved_game_instances WHERE user_id = $1",
        user_id
    )
    .execute(pool)
    .await?;

    Ok(())
}
