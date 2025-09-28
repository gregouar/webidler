use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use shared::data::{
    item::{ItemModifiers, ItemSlot},
    passive::PassivesTreeAscension,
    player::PlayerInventory,
    user::UserCharacterId,
};

use crate::{constants::CHARACTER_DATA_VERSION, db::pool::DbExecutor};

use super::utc_datetime::UtcDateTime;

#[derive(Debug, FromRow)]
#[allow(dead_code)]
pub struct CharacterDataEntry {
    pub character_id: UserCharacterId,

    pub data_version: String,
    pub inventory_data: Vec<u8>,
    pub passives_data: Option<Vec<u8>>,

    pub created_at: UtcDateTime,
    pub updated_at: UtcDateTime,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct InventoryData {
    pub equipped: HashMap<ItemSlot, ItemModifiers>,
    pub bag: Vec<ItemModifiers>,
    pub max_bag_size: u8,
}

pub async fn save_character_inventory<'c>(
    executor: impl DbExecutor<'c>,
    character_id: &UserCharacterId,
    inventory: &PlayerInventory,
) -> anyhow::Result<()> {
    let inventory_data = InventoryData {
        equipped: inventory
            .equipped_items()
            .map(|(item_slot, item_specs)| (item_slot, item_specs.modifiers.clone()))
            .collect(),
        bag: inventory
            .bag
            .iter()
            .map(|item_specs| item_specs.modifiers.clone())
            .collect(),
        max_bag_size: inventory.max_bag_size,
    };

    Ok(
        upsert_character_inventory_data(
            executor,
            character_id,
            rmp_serde::to_vec(&inventory_data)?,
        )
        .await?,
    )
}

pub(in crate::db) async fn upsert_character_inventory_data<'c>(
    executor: impl DbExecutor<'c>,
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
    .execute(executor)
    .await?;

    Ok(())
}

pub async fn save_character_passives<'c>(
    executor: impl DbExecutor<'c>,
    character_id: &UserCharacterId,
    passives: &PassivesTreeAscension,
) -> anyhow::Result<()> {
    Ok(
        upsert_character_passives_data(executor, character_id, rmp_serde::to_vec(passives)?)
            .await?,
    )
}

async fn upsert_character_passives_data<'c>(
    executor: impl DbExecutor<'c>,
    character_id: &UserCharacterId,
    passives_data: Vec<u8>,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE characters_data SET
            data_version = $2,
            passives_data = $3, 
            updated_at = CURRENT_TIMESTAMP
        WHERE character_id = $1",
        character_id,
        CHARACTER_DATA_VERSION,
        passives_data
    )
    .execute(executor)
    .await?;

    Ok(())
}

pub async fn load_character_data<'c>(
    executor: impl DbExecutor<'c>,
    character_id: &UserCharacterId,
) -> anyhow::Result<Option<(InventoryData, PassivesTreeAscension)>> {
    let character_data = read_character_data(executor, character_id).await?;
    if let Some(character_data) = character_data {
        Ok(Some((
            rmp_serde::from_slice(&character_data.inventory_data)?,
            character_data
                .passives_data
                .and_then(|passives_data| {
                    rmp_serde::from_slice::<PassivesTreeAscension>(&passives_data).ok()
                })
                .unwrap_or_default(),
        )))
    } else {
        Ok(None)
    }
}

async fn read_character_data<'c>(
    executor: impl DbExecutor<'c>,
    character_id: &UserCharacterId,
) -> Result<Option<CharacterDataEntry>, sqlx::Error> {
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
