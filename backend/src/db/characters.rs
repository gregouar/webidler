use sqlx::FromRow;

use shared::data::{
    area::AreaLevel,
    user::{UserCharacterId, UserId},
};

use super::{pool::DbPool, utc_datetime::UtcDateTime};

#[derive(Debug, FromRow)]
pub struct CharacterEntry {
    pub character_id: UserCharacterId,
    pub user_id: UserId,

    pub character_name: String,
    pub portrait: String,
    pub max_area_level: AreaLevel,
    pub resource_gems: f64,
    pub resource_shards: f64,

    pub created_at: UtcDateTime,
    pub updated_at: UtcDateTime,
    pub deleted_at: Option<UtcDateTime>,
}

#[derive(Debug, FromRow)]
pub struct CharacterAreaEntry {
    pub character_id: UserCharacterId,
    pub area_id: String,
    pub max_area_level: AreaLevel,

    pub created_at: UtcDateTime,
    pub updated_at: UtcDateTime,
}

pub async fn create_character(
    db_pool: &DbPool,
    user_id: &UserId,
    name: &str,
    portrait: &str,
) -> Result<UserCharacterId, sqlx::Error> {
    let character_id = uuid::Uuid::new_v4();

    sqlx::query!(
        r#"
        INSERT INTO characters (character_id,user_id, character_name, portrait)
        VALUES ($1, $2, $3, $4)
        "#,
        character_id,
        user_id,
        name,
        portrait
    )
    .execute(db_pool)
    .await?;

    Ok(character_id)
}

pub async fn read_character(
    db_pool: &DbPool,
    character_id: &UserCharacterId,
) -> Result<Option<CharacterEntry>, sqlx::Error> {
    Ok(sqlx::query_as!(
        CharacterEntry,
        r#"
        SELECT
            character_id as "character_id: UserCharacterId",
            user_id as "user_id: UserId",
            character_name,
            portrait,
            max_area_level as "max_area_level: AreaLevel",
            resource_gems,
            resource_shards,
            created_at,
            updated_at,
            deleted_at as "deleted_at: UtcDateTime"
         FROM characters WHERE character_id = $1
         "#,
        character_id
    )
    .fetch_optional(db_pool)
    .await?)
}

pub async fn read_character_areas_completed(
    db_pool: &DbPool,
    character_id: &UserCharacterId,
) -> Result<Vec<CharacterAreaEntry>, sqlx::Error> {
    Ok(sqlx::query_as!(
        CharacterAreaEntry,
        r#"
        SELECT
            character_id as "character_id: UserCharacterId",
            area_id,
            max_area_level as "max_area_level: AreaLevel",
            created_at,
            updated_at
         FROM character_area_completed WHERE character_id = $1
         "#,
        character_id
    )
    .fetch_all(db_pool)
    .await?)
}

pub async fn read_all_user_characters(
    db_pool: &DbPool,
    user_id: &UserId,
) -> Result<Vec<CharacterEntry>, sqlx::Error> {
    Ok(sqlx::query_as!(
        CharacterEntry,
        r#"
        SELECT
            character_id as "character_id: UserCharacterId",
            user_id as "user_id: UserId",
            character_name,
            portrait,
            max_area_level as "max_area_level: AreaLevel",
            resource_gems,
            resource_shards,
            created_at,
            updated_at,
            deleted_at as "deleted_at: UtcDateTime"
         FROM characters WHERE user_id = $1 AND deleted_at IS NULL
         "#,
        user_id
    )
    .fetch_all(db_pool)
    .await?)
}

pub async fn count_user_characters(db_pool: &DbPool, user_id: &UserId) -> Result<u8, sqlx::Error> {
    sqlx::query_scalar!(
        r#"
        SELECT
        COUNT(*) as "count!:u8" 
        FROM characters WHERE user_id = $1 AND deleted_at IS NULL
        "#,
        user_id
    )
    .fetch_one(db_pool)
    .await
}

pub async fn delete_character(
    db_pool: &DbPool,
    character_id: &UserCharacterId,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        UPDATE characters
        SET 
            deleted_at = CURRENT_TIMESTAMP,
            updated_at = CURRENT_TIMESTAMP 
        WHERE character_id = $1
        "#,
        character_id,
    )
    .execute(db_pool)
    .await?;

    Ok(())
}
