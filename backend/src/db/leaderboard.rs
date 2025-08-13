use sqlx::FromRow;

use shared::data::{
    area::AreaLevel,
    user::{UserCharacterId, UserId},
};

use super::{pool::DbPool, utc_datetime::UtcDateTime};

#[derive(Debug, FromRow)]
pub struct LeaderboardEntry {
    pub user_id: UserId,
    pub username: Option<String>,
    pub character_id: UserCharacterId,
    pub character_name: String,
    // pub portrait: String,
    pub area_id: String,
    pub area_level: AreaLevel,

    pub created_at: UtcDateTime,
}

pub async fn get_leaderboard(
    db_pool: &DbPool,
    limit: i64,
) -> Result<Vec<LeaderboardEntry>, sqlx::Error> {
    sqlx::query_as!(
        LeaderboardEntry,
        r#"
        SELECT
            users.user_id as "user_id: UserId",
            username,
            characters.character_id as "character_id: UserCharacterId",
            character_name,
            character_area_completed.area_id,
            character_area_completed.max_area_level as "area_level: AreaLevel",
            character_area_completed.updated_at as "created_at"
        FROM character_area_completed 
        INNER JOIN characters
        ON character_area_completed.character_id = characters.character_id
        INNER JOIN users
        ON characters.user_id = users.user_id
        ORDER BY character_area_completed.max_area_level DESC, character_area_completed.updated_at ASC
        LIMIT $1
        "#,
        limit
    )
    .fetch_all(db_pool)
    .await
}
