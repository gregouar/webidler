use sqlx::FromRow;

use shared::data::user::{UserCharacterId, UserId};

use super::{pool::DbPool, utc_datetime::UtcDateTime};

#[derive(Debug, FromRow)]
pub struct LeaderboardEntry {
    pub user_id: UserId,
    pub username: Option<String>,
    pub character_id: UserCharacterId,
    pub character_name: String,
    // pub portrait: String,
    pub area_id: String,
    pub area_level: i32,

    pub created_at: UtcDateTime,
}

pub async fn get_leaderboard(
    db_pool: &DbPool,
    top_n: i64,
) -> Result<Vec<LeaderboardEntry>, sqlx::Error> {
    sqlx::query_as!(
        LeaderboardEntry,
        r#"
        SELECT
            users.user_id           AS "user_id: UserId",
            username,
            characters.character_id AS "character_id: UserCharacterId",
            character_name,
            ranked.area_id,
            ranked.max_area_level   AS "area_level: i32",
            ranked.updated_at       AS "created_at"
        FROM (
            SELECT
                cac.*,
                ROW_NUMBER() OVER (
                    PARTITION BY cac.area_id
                    ORDER BY cac.max_area_level DESC, cac.updated_at ASC
                ) AS area_rank
            FROM character_area_completed cac
        ) AS ranked
        INNER JOIN characters ON ranked.character_id = characters.character_id
        INNER JOIN users ON characters.user_id = users.user_id
        WHERE ranked.area_rank <= $1
        ORDER BY ranked.area_id, ranked.area_rank;
        "#,
        top_n
    )
    .fetch_all(db_pool)
    .await
}

// pub async fn get_leaderboard(
//     db_pool: &DbPool,
//     limit: i64,
// ) -> Result<Vec<LeaderboardEntry>, sqlx::Error> {
//     sqlx::query_as!(
//         LeaderboardEntry,
//         r#"
//         SELECT
//             users.user_id as "user_id: UserId",
//             username,
//             characters.character_id as "character_id: UserCharacterId",
//             character_name,
//             character_area_completed.area_id,
//             character_area_completed.max_area_level as "area_level: i32",
//             character_area_completed.updated_at as "created_at"
//         FROM character_area_completed
//         INNER JOIN characters
//         ON character_area_completed.character_id = characters.character_id
//         INNER JOIN users
//         ON characters.user_id = users.user_id
//         ORDER BY character_area_completed.max_area_level DESC, character_area_completed.updated_at ASC
//         LIMIT $1
//         "#,
//         limit
//     )
//     .fetch_all(db_pool)
//     .await
// }
