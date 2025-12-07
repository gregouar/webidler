use shared::data::{
    stash::{StashId, StashType},
    user::{UserCharacterId, UserId},
};
use sqlx::{types::Json, FromRow};

use crate::db::pool::DbExecutor;

// pub type DbStashType

#[derive(Debug, FromRow)]
pub struct StashEntry {
    pub stash_id: StashId,
    pub user_id: UserId,

    pub stash_type: Json<StashType>,
    pub title: Option<String>,

    pub items_amount: i64,
    pub max_items: i32,
    pub resource_gems: f64,
}

pub async fn create_stash<'c>(
    executor: impl DbExecutor<'c>,
    user_id: UserId,
    stash_type: StashType,
    max_items: i32,
    title: &str,
) -> Result<StashEntry, sqlx::Error> {
    let stash_id = uuid::Uuid::new_v4();

    let stash_type = Json(stash_type);
    sqlx::query!(
        r#"
        INSERT INTO stashes (stash_id, user_id, stash_type, title, max_items)
        VALUES ($1, $2, $3, $4, $5)
        "#,
        stash_id,
        user_id,
        stash_type,
        title,
        max_items
    )
    .execute(executor)
    .await?;

    Ok(StashEntry {
        stash_id,
        user_id,
        stash_type,
        title: Some(title.to_string()),
        items_amount: 0,
        max_items,
        resource_gems: 0.0,
    })
}

pub async fn get_stash<'c>(
    executor: impl DbExecutor<'c>,
    stash_id: &StashId,
) -> Result<Option<StashEntry>, sqlx::Error> {
    sqlx::query_as!(
        StashEntry,
        r#"
        SELECT
            stash_id as "stash_id: StashId",
            user_id as "user_id: UserId",
            stash_type as "stash_type: Json<StashType>",
            title as "title?",
            max_items,
            resource_gems,
            (
                SELECT
                    COUNT(1) AS "count!"
                FROM stash_items
                WHERE deleted_at IS NULL
                AND stash_items.stash_id = stashes.stash_id
            ) as "items_amount!: i64"
        FROM stashes
        WHERE stash_id = $1 AND deleted_at IS NULL
        "#,
        stash_id
    )
    .fetch_optional(executor)
    .await
}

pub async fn get_character_stash_by_type<'c>(
    executor: impl DbExecutor<'c>,
    character_id: &UserCharacterId,
    stash_type: StashType,
) -> Result<Option<StashEntry>, sqlx::Error> {
    let stash_type = Json(stash_type);
    sqlx::query_as!(
        StashEntry,
        r#"
        SELECT
            stashes.stash_id as "stash_id: StashId",
            stashes.user_id as "user_id: UserId",
            stashes.stash_type as "stash_type: Json<StashType>",
            stashes.title as "title?",
            stashes.max_items,
            stashes.resource_gems,
            (
                SELECT
                    COUNT(1) AS "count!"
                FROM stash_items
                WHERE deleted_at IS NULL
                AND stash_items.stash_id = stashes.stash_id
            ) as "items_amount!: i64"
        FROM stashes
        INNER JOIN characters ON characters.user_id = stashes.user_id
        WHERE 
            character_id = $1
            AND stash_type = $2 
            AND stashes.deleted_at IS NULL
        "#,
        character_id,
        stash_type
    )
    .fetch_optional(executor)
    .await
}

pub async fn update_stash_size<'c>(
    executor: impl DbExecutor<'c>,
    stash_id: &StashId,
    max_items: usize,
) -> Result<(), sqlx::Error> {
    let max_items = max_items as i32;

    sqlx::query!(
        r#"
        UPDATE stashes
        SET 
            max_items =  $2,
            updated_at = CURRENT_TIMESTAMP 
        WHERE stash_id = $1
        "#,
        stash_id,
        max_items,
    )
    .execute(executor)
    .await?;

    Ok(())
}
