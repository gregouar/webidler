use shared::data::{
    realms::RealmId,
    stash::{StashId, StashType},
    user::UserId,
};
use sqlx::{FromRow, types::Json};

use crate::db::{characters::CharacterEntry, pool::DbExecutor};

// pub type DbStashType

#[derive(Debug, FromRow)]
pub struct StashEntry {
    pub stash_id: StashId,
    pub user_id: UserId,
    pub owner_id: uuid::Uuid,

    pub realm_id: RealmId,

    pub stash_type: Json<StashType>,
    pub title: Option<String>,

    pub items_amount: i64,
    pub max_items: i64,
    pub resource_gems: f64,
}

pub async fn create_stash<'c>(
    executor: impl DbExecutor<'c>,
    user_id: UserId,
    owner_id: uuid::Uuid,
    realm_id: RealmId,
    stash_type: StashType,
    max_items: i64,
    title: &str,
) -> Result<StashEntry, sqlx::Error> {
    let stash_id = uuid::Uuid::new_v4();

    let stash_type = Json(stash_type);
    let stash_type_value = serde_json::to_value(stash_type).unwrap();
    sqlx::query!(
        r#"
        INSERT INTO stashes (stash_id, user_id, owner_id, stash_type, title, max_items, realm_id)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
        stash_id,
        user_id,
        owner_id,
        stash_type_value,
        title,
        max_items,
        realm_id
    )
    .execute(executor)
    .await?;

    Ok(StashEntry {
        stash_id,
        user_id,
        owner_id,
        realm_id,
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
    realm_id: &RealmId,
) -> Result<Option<StashEntry>, sqlx::Error> {
    sqlx::query_as!(
        StashEntry,
        r#"
        SELECT
            stash_id as "stash_id: StashId",
            user_id as "user_id: UserId",
            owner_id as "owner_id: uuid::Uuid",
            realm_id as "realm_id!",
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
        WHERE 
            stash_id = $1 
            AND realm_id = $2
            AND deleted_at IS NULL 
        "#,
        stash_id,
        realm_id
    )
    .fetch_optional(executor)
    .await
}

pub async fn get_character_stash_by_type<'c>(
    executor: impl DbExecutor<'c>,
    character: &CharacterEntry,
    stash_type: StashType,
) -> Result<Option<StashEntry>, sqlx::Error> {
    let owner_id = match stash_type {
        StashType::Character => character.character_id,
        StashType::User | StashType::Market => character.user_id,
    };
    let stash_type = serde_json::to_value(Json(stash_type)).unwrap();
    sqlx::query_as!(
        StashEntry,
        r#"
        SELECT
            stashes.stash_id as "stash_id: StashId",
            stashes.user_id as "user_id: UserId",
            stashes.owner_id as "owner_id: uuid::Uuid",
            stashes.realm_id as "realm_id!",
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
        WHERE 
            owner_id = $1
            AND stash_type = $2
            AND realm_id = $3
            AND stashes.deleted_at IS NULL
        "#,
        owner_id,
        stash_type,
        character.realm_id
    )
    .fetch_optional(executor)
    .await
}

pub async fn update_stash_size<'c>(
    executor: impl DbExecutor<'c>,
    stash_id: &StashId,
    max_items: usize,
) -> Result<(), sqlx::Error> {
    let max_items = max_items as i64;

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
pub async fn update_stash_gems<'c>(
    executor: impl DbExecutor<'c>,
    stash_id: &StashId,
    gems_difference: f64,
) -> Result<f64, sqlx::Error> {
    sqlx::query_scalar!(
        r#"
        UPDATE stashes
        SET 
            resource_gems = resource_gems + $2,
            updated_at = CURRENT_TIMESTAMP 
        WHERE stash_id = $1
        RETURNING 
            resource_gems
        "#,
        stash_id,
        gems_difference,
    )
    .fetch_one(executor)
    .await
}
