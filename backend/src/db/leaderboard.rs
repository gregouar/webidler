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
    pub elapsed_time: Option<f64>,
}

// TODO: Rework for new leaderboard
pub async fn get_leaderboard(
    db_pool: &DbPool,
    top_n: i64,
) -> Result<Vec<LeaderboardEntry>, sqlx::Error> {
    sqlx::query_as!(
        LeaderboardEntry,
        r#"
        WITH best_runs AS (
            SELECT
                gs.*,
                ROW_NUMBER() OVER (
                    PARTITION BY gs.area_id, gs.character_id
                    ORDER BY gs.area_level DESC, gs.elapsed_time ASC, gs.created_at ASC
                ) AS best_rank
            FROM game_stats gs
            WHERE gs.data_version >= '0.1.9' OR gs.data_version = '0.1.8' OR gs.data_version = '0.1.8_0.1.7'
        ),
        leaderboard AS (
            SELECT
                br.*,
                ROW_NUMBER() OVER (
                    PARTITION BY br.area_id
                    ORDER BY br.area_level DESC, br.elapsed_time ASC, br.created_at ASC
                ) AS area_rank
            FROM best_runs br
            WHERE br.best_rank = 1
              AND (br.elapsed_time IS NULL OR br.elapsed_time <> 0)
              AND area_level <> 0
        )
        SELECT
            u.user_id           AS "user_id: UserId",
            u.username,
            c.character_id      AS "character_id: UserCharacterId",
            c.character_name,
            lb.area_id,
            lb.area_level       AS "area_level: i32",
            lb.created_at       AS "created_at",
            lb.elapsed_time     AS "elapsed_time?"
        FROM leaderboard lb
        JOIN characters c ON lb.character_id = c.character_id
        JOIN users u      ON c.user_id = u.user_id
        WHERE lb.area_rank <= $1
        ORDER BY lb.area_id, lb.area_rank;
        "#,
        top_n
    )
    .fetch_all(db_pool)
    .await
}
