use std::collections::{HashMap, HashSet};

use anyhow;

use sqlx::{types::JsonValue, FromRow};

use shared::data::{item::ItemSlot, player::EquippedSlot, user::UserCharacterId};

use crate::{
    constants::DATA_VERSION, db::utc_datetime::UtcDateTime, game::game_data::GameInstanceData,
};

use super::pool::DbExecutor;

#[derive(Debug, FromRow)]
pub struct GameStatsEntry {
    pub character_id: UserCharacterId,

    pub area_id: String,
    pub area_level: i32,
    pub elapsed_time: Option<f64>,

    pub stats_data: Option<JsonValue>,
    pub items_data: Option<JsonValue>,
    pub passives_data: Option<JsonValue>,
    pub skills_data: Option<JsonValue>,
    pub data_version: Option<String>,

    pub created_at: UtcDateTime,
}

pub async fn save_game_stats<'c>(
    executor: impl DbExecutor<'c>,
    character_id: &UserCharacterId,
    game_instance_data: &GameInstanceData,
) -> anyhow::Result<()> {
    Ok(insert_game_stats(
        executor,
        character_id,
        &game_instance_data.area_id.clone(),
        game_instance_data.area_state.read().area_level as i32,
        game_instance_data
            .game_stats
            .elapsed_time_at_max_level
            .as_secs_f64(),
        serde_json::to_value(&game_instance_data.game_stats)?,
        serde_json::to_value(&game_instance_data.player_inventory.read().equipped)?,
        serde_json::to_value(game_instance_data.passives_tree_state.read())?,
        serde_json::to_value(&game_instance_data.player_specs.read().bought_skills)?,
    )
    .await?)
}

async fn insert_game_stats<'c>(
    executor: impl DbExecutor<'c>,
    character_id: &UserCharacterId,
    area_id: &str,
    area_level: i32,
    elapsed_time: f64,
    stats_data: JsonValue,
    items_data: JsonValue,
    passives_data: JsonValue,
    skills_data: JsonValue,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO game_stats 
            (character_id, area_id, area_level, elapsed_time, data_version, stats_data, items_data, passives_data, skills_data) 
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        ",
        character_id,
        area_id,
        area_level,
        elapsed_time,
        DATA_VERSION,
        stats_data,
        items_data,
        passives_data,
        skills_data,
    )
    .execute(executor)
    .await?;

    Ok(())
}

pub async fn load_last_game_stats<'c>(
    executor: impl DbExecutor<'c>,
    character_id: &UserCharacterId,
) -> anyhow::Result<
    Option<(
        Option<HashMap<ItemSlot, EquippedSlot>>,
        Option<HashSet<String>>,
    )>,
> {
    let game_stats_data = read_last_game_stats(executor, character_id).await?;
    if let Some(game_stats_data) = game_stats_data {
        Ok(Some((
            game_stats_data
                .items_data
                .and_then(|data| serde_json::from_value(data).ok()),
            game_stats_data
                .skills_data
                .and_then(|data| serde_json::from_value(data).ok()),
        )))
    } else {
        Ok(None)
    }
}

async fn read_last_game_stats<'c>(
    executor: impl DbExecutor<'c>,
    character_id: &UserCharacterId,
) -> Result<Option<GameStatsEntry>, sqlx::Error> {
    sqlx::query_as!(
        GameStatsEntry,
        r#"
        SELECT
            character_id as "character_id: UserCharacterId",
            area_id,
            area_level as "area_level: i32",
            elapsed_time as "elapsed_time?",
            stats_data as "stats_data?: JsonValue",
            items_data as "items_data?: JsonValue",
            passives_data as "passives_data?: JsonValue",
            skills_data as "skills_data?: JsonValue",
            data_version,
            created_at
         FROM game_stats WHERE character_id = $1
         ORDER BY created_at DESC
         LIMIT 1
         "#,
        character_id
    )
    .fetch_optional(executor)
    .await
}
