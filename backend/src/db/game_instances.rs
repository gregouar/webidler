use anyhow;

use sqlx::{Executor, FromRow};

use shared::data::user::UserCharacterId;

use crate::{
    constants::CHARACTER_DATA_VERSION,
    db::utc_datetime::UtcDateTime,
    game::{data::master_store, game_data::GameInstanceData},
};

use super::pool::Database;

#[derive(Debug, FromRow)]
pub struct SavedGameInstance {
    pub character_id: UserCharacterId,

    pub area_id: String,
    pub area_level: i32,

    pub saved_at: UtcDateTime,
    pub data_version: String,
    pub game_data: Vec<u8>, // Assuming game_data is stored as a binary blob
}

pub async fn save_game_instance_data<'c, E>(
    executor: E,
    character_id: &UserCharacterId,
    game_instance_data: GameInstanceData,
) -> anyhow::Result<()>
where
    E: Executor<'c, Database = Database>,
{
    Ok(upsert_saved_game_instance(
        executor,
        character_id,
        &game_instance_data.area_id.clone(),
        game_instance_data.area_state.read().area_level as i32,
        game_instance_data.to_bytes()?,
    )
    .await?)
}

async fn upsert_saved_game_instance<'c, E>(
    executor: E,
    character_id: &UserCharacterId,
    area_id: &str,
    area_level: i32,
    game_data: Vec<u8>,
) -> Result<(), sqlx::Error>
where
    E: Executor<'c, Database = Database>,
{
    sqlx::query!(
        "INSERT INTO saved_game_instances 
            (character_id, area_id, area_level, data_version, game_data) 
            VALUES ($1, $2, $3, $4, $5)
         ON CONFLICT(character_id) DO UPDATE SET 
            area_id = EXCLUDED.area_id, 
            area_level = EXCLUDED.area_level, 
            game_data = EXCLUDED.game_data, 
            data_version = EXCLUDED.data_version, 
            saved_at = CURRENT_TIMESTAMP",
        character_id,
        area_id,
        area_level,
        CHARACTER_DATA_VERSION,
        game_data
    )
    .execute(executor)
    .await?;

    Ok(())
}

pub async fn load_game_instance_data<'c, E>(
    executor: E,
    master_store: &master_store::MasterStore,
    character_id: &UserCharacterId,
) -> anyhow::Result<Option<GameInstanceData>>
where
    E: Executor<'c, Database = Database>,
{
    let saved_game_instance = load_saved_game_instance(executor, character_id).await?;
    if let Some(instance) = saved_game_instance {
        Ok(Some(GameInstanceData::from_bytes(
            master_store,
            &instance.game_data,
        )?))
    } else {
        Ok(None)
    }
}

async fn load_saved_game_instance<'c, E>(
    executor: E,
    character_id: &UserCharacterId,
) -> Result<Option<SavedGameInstance>, sqlx::Error>
where
    E: Executor<'c, Database = Database>,
{
    let instance = sqlx::query_as!(
        SavedGameInstance,
        r#"SELECT 
                character_id as "character_id: UserCharacterId", 
                area_id, 
                area_level as "area_level: i32", 
                saved_at, 
                data_version, 
                game_data 
            FROM saved_game_instances 
            WHERE character_id = $1"#,
        character_id
    )
    .fetch_optional(executor)
    .await?;

    Ok(instance)
}

pub async fn delete_game_instance_data<'c, E>(
    executor: E,
    character_id: &UserCharacterId,
) -> Result<(), sqlx::Error>
where
    E: Executor<'c, Database = Database>,
{
    sqlx::query!(
        "DELETE FROM saved_game_instances WHERE character_id = $1",
        character_id
    )
    .execute(executor)
    .await?;

    Ok(())
}
