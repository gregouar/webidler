use sqlx::{FromRow, Transaction};

use shared::data::user::UserId;

use super::{
    pool::{Database, DbPool},
    utc_datetime::UtcDateTime,
};

#[derive(Debug, FromRow)]
pub struct PasswordResetEntry {
    pub token_id: i64,
    pub user_id: UserId,
}

pub async fn create_password_reset(
    db_pool: &DbPool,
    user_id: &UserId,
    token_hash: &[u8],
    expires_at: &UtcDateTime,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE password_reset_tokens SET
        expires_at = CURRENT_TIMESTAMP
        WHERE user_id = $1 AND expires_at > CURRENT_TIMESTAMP",
        user_id
    )
    .execute(db_pool)
    .await?;

    sqlx::query!(
        r#"
        INSERT INTO password_reset_tokens (user_id, token_hash, expires_at)
        VALUES ($1, $2, $3)
        "#,
        user_id,
        token_hash,
        expires_at
    )
    .execute(db_pool)
    .await?;

    Ok(())
}

pub async fn redeem_password_reset<'c>(
    executor: &mut Transaction<'c, Database>,
    user_id: &UserId,
    token_hash: &[u8],
) -> Result<Option<PasswordResetEntry>, sqlx::Error> {
    sqlx::query_as!(
        PasswordResetEntry,
        r#"
            UPDATE password_reset_tokens
            SET used_at = CURRENT_TIMESTAMP
            WHERE
                user_id = $1
                AND token_hash = $2
                AND expires_at > CURRENT_TIMESTAMP
                AND used_at IS NULL
            RETURNING
                token_id,
                user_id as "user_id: UserId"
        "#,
        user_id,
        token_hash
    )
    .fetch_optional(&mut **executor)
    .await
}
