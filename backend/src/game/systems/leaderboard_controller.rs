use anyhow::Result;

use crate::{
    db,
    game::{game_data::GameInstanceData, sessions::SessionId},
};

pub async fn save_game_score(
    db_pool: &db::DbPool,
    session_id: &SessionId,
    game_data: &GameInstanceData,
) -> Result<()> {
    db::leaderboard::upsert_leaderboard_entry(
        db_pool,
        session_id,
        &game_data.player_specs.read().character_specs.name,
        game_data.game_stats.highest_area_level,
        game_data.game_stats.elapsed_time,
        &format!(
            "Player level: {}, Player deaths: {}",
            game_data.player_specs.read().level,
            game_data.game_stats.player_deaths
        ),
    )
    .await?;
    Ok(())
}
