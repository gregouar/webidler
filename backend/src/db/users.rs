use chrono::{DateTime, Utc};
use sqlx::FromRow;

use shared::data::user::UserId;

use super::{pool::DbPool, utc_datetime::UtcDateTime};

#[derive(Debug, FromRow)]
pub struct UserEntry {
    pub user_id: UserId,
    pub username: Option<String>,
    pub email_crypt: Option<Vec<u8>>,
    pub terms_accepted_at: UtcDateTime,
    pub is_admin: bool,
    pub max_characters: i16,
    pub last_login_at: Option<UtcDateTime>,
    pub created_at: UtcDateTime,
    pub updated_at: UtcDateTime,
    pub deleted_at: Option<UtcDateTime>,
}

pub async fn create_user(
    db_pool: &DbPool,
    username: &str,
    email_crypt: Option<&[u8]>,
    email_hash: Option<&[u8]>,
    password_hash: &str,
    terms_accepted_at: &DateTime<Utc>,
    max_characters: i16,
) -> Result<Option<uuid::Uuid>, sqlx::Error> {
    let user_id = uuid::Uuid::new_v4();

    let res = sqlx::query!(
        r#"
        INSERT INTO users (user_id, username, email_crypt, email_hash, password_hash, terms_accepted_at, max_characters)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
        user_id,
        username,
        email_crypt,
        email_hash,
        password_hash,
        terms_accepted_at,
        max_characters
    )
    .execute(db_pool)
    .await;

    match res {
        Ok(_) => Ok(Some(user_id)),
        Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => Ok(None),
        Err(e) => Err(e),
    }
}

pub async fn read_user(
    db_pool: &DbPool,
    user_id: &UserId,
) -> Result<Option<UserEntry>, sqlx::Error> {
    sqlx::query_as!(
        UserEntry,
        r#"
        SELECT 
            user_id as "user_id: UserId", 
            username,
            email_crypt, 
            terms_accepted_at, 
            is_admin, 
            max_characters as "max_characters!: i16", 
            last_login_at as "last_login_at?: UtcDateTime",
            created_at, 
            updated_at, 
            deleted_at as "deleted_at?: UtcDateTime"
         FROM users WHERE user_id = $1
         "#,
        user_id
    )
    .fetch_optional(db_pool)
    .await
}

pub async fn auth_user(
    db_pool: &DbPool,
    username: &str,
) -> Result<Option<(UserId, Option<String>)>, sqlx::Error> {
    Ok(sqlx::query!(
        r#"
        SELECT 
            user_id as "user_id: UserId", 
            password_hash 
        FROM users WHERE LOWER(username) = LOWER($1)
        "#,
        username
    )
    .fetch_optional(db_pool)
    .await?
    .map(|record| (record.user_id, record.password_hash)))
}

pub async fn update_last_login(db_pool: &DbPool, user_id: &UserId) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
            UPDATE users
            SET last_login_at = CURRENT_TIMESTAMP
            WHERE user_id = $1
        "#,
        user_id,
    )
    .execute(db_pool)
    .await?;
    Ok(())
}

// pub async fn delete_user(
//     db_pool: &DbPool,
//     user_id: &UserId,
// ) -> Result<Option<UserEntry>, sqlx::Error> {
//     Ok(sqlx::query_as!(
//         UserEntry,
//         r#"
//         SELECT
//             user_id as "user_id: UserId",
//             username, email, terms_accepted_at, is_admin,
//             max_characters as "max_characters: u8",
//             last_login_at as "last_login_at: UtcDateTime",
//             created_at, updated_at,
//             deleted_at as "deleted_at: UtcDateTime"
//          FROM users WHERE user_id = $1
//          "#,
//         user_id
//     )
//     .fetch_optional(db_pool)
//     .await?)
// }
