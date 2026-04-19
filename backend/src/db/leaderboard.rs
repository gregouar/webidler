use sqlx::FromRow;

use shared::data::{
    realms::RealmId,
    user::{UserCharacterId, UserId},
};

use crate::constants::DATA_VERSION;

use super::{
    pool::{Database, DbPool},
    utc_datetime::UtcDateTime,
};

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
    pub elapsed_time: f64,
}

pub async fn get_leaderboard(
    db_pool: &DbPool,
    top_n: i64,
    realm_id: &RealmId,
) -> Result<Vec<LeaderboardEntry>, sqlx::Error> {
    sqlx::query_as!(
        LeaderboardEntry,
        r#"
        WITH ranked AS (
            SELECT
                lb.character_id,
                lb.area_id,
                lb.area_level,
                lb.elapsed_time,
                lb.updated_at,
                ROW_NUMBER() OVER (
                    PARTITION BY lb.area_id
                    ORDER BY lb.area_level DESC, lb.elapsed_time ASC, lb.updated_at ASC
                ) AS area_rank
            FROM leaderboard lb
            WHERE lb.realm_id = $2
        )
        SELECT
            u.user_id           AS "user_id: UserId",
            u.username,
            c.character_id      AS "character_id: UserCharacterId",
            c.character_name,
            r.area_id,
            r.area_level        AS "area_level: i32",
            r.updated_at        AS "created_at",
            r.elapsed_time
        FROM ranked r
        JOIN characters c ON r.character_id = c.character_id
        JOIN users u      ON c.user_id = u.user_id
        WHERE r.area_rank <= $1
        ORDER BY r.area_id, r.area_rank;
        "#,
        top_n,
        realm_id
    )
    .fetch_all(db_pool)
    .await
}

/// Return whether the result is a new global high score in the realm
pub async fn update_leaderboard<'c>(
    executor: &mut sqlx::Transaction<'c, Database>,
    character_id: &UserCharacterId,
    realm_id: &RealmId,
    area_id: &str,
    area_level: i32,
    elapsed_time: f64,
) -> Result<bool, sqlx::Error> {
    let count = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) as "count!"
        FROM leaderboard lb
        WHERE lb.realm_id = $1
          AND lb.area_id = $2
          AND lb.area_level >= $3
        "#,
        realm_id,
        area_id,
        area_level
    )
    .fetch_one(&mut **executor)
    .await?;

    sqlx::query!(
        r#"
        INSERT INTO leaderboard (
            character_id,
            realm_id,
            area_id,
            area_level,
            elapsed_time,
            data_version
        )
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT(character_id, realm_id, area_id) DO UPDATE SET
            area_level = EXCLUDED.area_level,
            elapsed_time = EXCLUDED.elapsed_time,
            data_version = EXCLUDED.data_version,
            updated_at = CURRENT_TIMESTAMP
        WHERE
            EXCLUDED.area_level > leaderboard.area_level
            OR (
                EXCLUDED.area_level = leaderboard.area_level
                AND (
                    leaderboard.elapsed_time IS NULL
                    OR EXCLUDED.elapsed_time < leaderboard.elapsed_time
                )
            )
        "#,
        character_id,
        realm_id,
        area_id,
        area_level,
        elapsed_time,
        DATA_VERSION
    )
    .execute(&mut **executor)
    .await?;

    Ok(count == 0)
}
