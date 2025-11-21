use anyhow;

use sqlx::{types::JsonValue, FromRow};

use shared::data::user::UserCharacterId;

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
        serde_json::to_value(&game_instance_data.passives_tree_state.read())?,
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
