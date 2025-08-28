use sqlx::{Executor, FromRow};

use shared::data::{
    passive::PassivesTreeAscension, player::PlayerInventory, user::UserCharacterId,
};

use crate::{constants::CHARACTER_DATA_VERSION, db::pool::Database};

use super::utc_datetime::UtcDateTime;

#[derive(Debug, FromRow)]
#[allow(dead_code)]
struct CharacterDataEntry {
    pub character_id: UserCharacterId,

    pub data_version: String,
    pub inventory_data: Vec<u8>,
    pub passives_data: Option<Vec<u8>>,

    pub created_at: UtcDateTime,
    pub updated_at: UtcDateTime,
}

pub async fn save_character_inventory<'c, E>(
    executor: E,
    character_id: &UserCharacterId,
    inventory: &PlayerInventory,
) -> anyhow::Result<()>
where
    E: Executor<'c, Database = Database>,
{
    Ok(
        upsert_character_inventory_data(executor, character_id, rmp_serde::to_vec(inventory)?)
            .await?,
    )
}

async fn upsert_character_inventory_data<'c, E>(
    executor: E,
    character_id: &UserCharacterId,
    inventory_data: Vec<u8>,
) -> Result<(), sqlx::Error>
where
    E: Executor<'c, Database = Database>,
{
    sqlx::query!(
        "INSERT INTO characters_data (character_id, data_version, inventory_data) VALUES ($1, $2, $3)
         ON CONFLICT(character_id) DO UPDATE SET 
            data_version = $2,
            inventory_data = EXCLUDED.inventory_data, 
            updated_at = CURRENT_TIMESTAMP",
        character_id,
        CHARACTER_DATA_VERSION,
        inventory_data
    )
    .execute(executor)
    .await?;

    Ok(())
}

pub async fn save_character_passives<'c, E>(
    executor: E,
    character_id: &UserCharacterId,
    passives: &PassivesTreeAscension,
) -> anyhow::Result<()>
where
    E: Executor<'c, Database = Database>,
{
    Ok(
        upsert_character_passives_data(executor, character_id, rmp_serde::to_vec(passives)?)
            .await?,
    )
}

async fn upsert_character_passives_data<'c, E>(
    executor: E,
    character_id: &UserCharacterId,
    passives_data: Vec<u8>,
) -> Result<(), sqlx::Error>
where
    E: Executor<'c, Database = Database>,
{
    sqlx::query!(
        "INSERT INTO characters_data (character_id, data_version, passives_data) VALUES ($1, $2, $3)
         ON CONFLICT(character_id) DO UPDATE SET 
            data_version = $2,
            passives_data = EXCLUDED.passives_data, 
            updated_at = CURRENT_TIMESTAMP",
        character_id,
        CHARACTER_DATA_VERSION,
        passives_data
    )
    .execute(executor)
    .await?;

    Ok(())
}

pub async fn load_character_data<'c, E>(
    executor: E,
    character_id: &UserCharacterId,
) -> anyhow::Result<Option<(PlayerInventory, PassivesTreeAscension)>>
where
    E: Executor<'c, Database = Database>,
{
    let character_data = read_character_data(executor, character_id).await?;
    if let Some(character_data) = character_data {
        Ok(Some((
            rmp_serde::from_slice::<PlayerInventory>(&character_data.inventory_data)?,
            character_data
                .passives_data
                .map(|passives_data| {
                    rmp_serde::from_slice::<PassivesTreeAscension>(&passives_data).ok()
                })
                .flatten()
                .unwrap_or_default(),
        )))
    } else {
        Ok(None)
    }
}

async fn read_character_data<'c, E>(
    executor: E,
    character_id: &UserCharacterId,
) -> Result<Option<CharacterDataEntry>, sqlx::Error>
where
    E: Executor<'c, Database = Database>,
{
    sqlx::query_as!(
        CharacterDataEntry,
        r#"
        SELECT
            character_id as "character_id: UserCharacterId",
            data_version,
            inventory_data,
            passives_data,
            created_at,
            updated_at
         FROM characters_data WHERE character_id = $1
         "#,
        character_id
    )
    .fetch_optional(executor)
    .await
}
