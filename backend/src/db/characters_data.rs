use sqlx::FromRow;

use shared::data::{player::PlayerInventory, user::UserCharacterId};

use crate::constants::CHARACTER_DATA_VERSION;

use super::{pool::DbPool, utc_datetime::UtcDateTime};

#[derive(Debug, FromRow)]
#[allow(dead_code)]
struct CharacterDataEntry {
    pub character_id: UserCharacterId,

    pub data_version: String,
    pub inventory_data: Vec<u8>,

    pub created_at: UtcDateTime,
    pub updated_at: UtcDateTime,
}

pub async fn save_character_data(
    pool: &DbPool,
    character_id: &UserCharacterId,
    inventory: &PlayerInventory,
) -> anyhow::Result<()> {
    Ok(upsert_character_data(pool, character_id, rmp_serde::to_vec(inventory)?).await?)
}

async fn upsert_character_data(
    pool: &DbPool,
    character_id: &UserCharacterId,
    inventory_data: Vec<u8>,
) -> Result<(), sqlx::Error> {
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
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn load_character_data(
    db_pool: &DbPool,
    character_id: &UserCharacterId,
) -> anyhow::Result<Option<PlayerInventory>> {
    let character_data = read_character_data(db_pool, character_id).await?;
    if let Some(character_data) = character_data {
        Ok(Some(rmp_serde::from_slice::<PlayerInventory>(
            &character_data.inventory_data,
        )?))
    } else {
        Ok(None)
    }
}

async fn read_character_data(
    db_pool: &DbPool,
    character_id: &UserCharacterId,
) -> Result<Option<CharacterDataEntry>, sqlx::Error> {
    Ok(sqlx::query_as!(
        CharacterDataEntry,
        r#"
        SELECT
            character_id as "character_id: UserCharacterId",
            data_version,
            inventory_data,
            created_at,
            updated_at
         FROM characters_data WHERE character_id = $1
         "#,
        character_id
    )
    .fetch_optional(db_pool)
    .await?)
}
