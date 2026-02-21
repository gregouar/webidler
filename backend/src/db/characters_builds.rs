use sqlx::FromRow;

use shared::data::{passive::PurchasedNodes, user::UserCharacterId};

use crate::{constants::DATA_VERSION, db::pool::DbExecutor};

use super::utc_datetime::UtcDateTime;

// TODO: Save last skill used here instead?

#[derive(Debug, FromRow)]
#[allow(dead_code)]
pub struct CharacterBuildEntry {
    pub character_id: UserCharacterId,

    pub passives_data: Option<Vec<u8>>,

    pub data_version: String,

    pub created_at: UtcDateTime,
    pub updated_at: UtcDateTime,
}

pub async fn save_character_passives_build<'c>(
    executor: impl DbExecutor<'c>,
    character_id: &UserCharacterId,
    purchased_nodes: &PurchasedNodes,
) -> anyhow::Result<()> {
    Ok(upsert_character_passives_build_data(
        executor,
        character_id,
        "default",
        rmp_serde::to_vec(&purchased_nodes)?,
    )
    .await?)
}

pub(in crate::db) async fn upsert_character_passives_build_data<'c>(
    executor: impl DbExecutor<'c>,
    character_id: &UserCharacterId,
    title: &'static str,
    inventory_data: Vec<u8>,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO characters_builds (character_id, title, data_version, passives_data) VALUES ($1, $2, $3, $4)
         ON CONFLICT(character_id, title) DO UPDATE SET 
            data_version = $2,
            passives_data = EXCLUDED.passives_data, 
            updated_at = CURRENT_TIMESTAMP",
        character_id,
        title,
        DATA_VERSION,
        inventory_data
    )
    .execute(executor)
    .await?;

    Ok(())
}

pub async fn load_character_build<'c>(
    executor: impl DbExecutor<'c>,
    character_id: &UserCharacterId,
) -> anyhow::Result<Option<PurchasedNodes>> {
    let character_build_data = read_character_build_data(executor, character_id, "default").await?;

    if let Some(character_build_data) = character_build_data {
        Ok(Some(
            character_build_data
                .passives_data
                .and_then(|passives_data| {
                    rmp_serde::from_slice::<PurchasedNodes>(&passives_data).ok()
                })
                .unwrap_or_default(),
        ))
    } else {
        Ok(None)
    }
}

async fn read_character_build_data<'c>(
    executor: impl DbExecutor<'c>,
    character_id: &UserCharacterId,
    title: &'static str,
) -> Result<Option<CharacterBuildEntry>, sqlx::Error> {
    sqlx::query_as!(
        CharacterBuildEntry,
        r#"
        SELECT
            character_id as "character_id: UserCharacterId",
            passives_data,
            data_version,
            created_at,
            updated_at
         FROM characters_builds WHERE character_id = $1 AND title = $2
         "#,
        character_id,
        title
    )
    .fetch_optional(executor)
    .await
}
