use sqlx::{FromRow, Transaction};

use shared::data::user::{UserCharacterId, UserId};

use crate::db::pool::Database;

use super::{pool::DbExecutor, utc_datetime::UtcDateTime};

#[derive(Debug, FromRow)]
pub struct CharacterEntry {
    pub character_id: UserCharacterId,
    pub user_id: UserId,

    pub character_name: String,
    pub portrait: String,
    pub max_area_level: i32,
    pub resource_gems: f64,
    pub resource_shards: f64,
    pub resource_gold: f64,

    pub created_at: UtcDateTime,
    pub updated_at: UtcDateTime,
    pub deleted_at: Option<UtcDateTime>,

    // Joined
    pub area_id: Option<String>,
    pub area_level: Option<i32>,
}

#[derive(Debug, FromRow)]
pub struct CharacterResources {
    pub resource_gems: f64,
    pub resource_shards: f64,
    pub resource_gold: f64,
}

#[derive(Debug, FromRow)]
pub struct CharacterAreaEntry {
    pub character_id: UserCharacterId,
    pub area_id: String,
    pub max_area_level: i32,

    pub created_at: UtcDateTime,
    pub updated_at: UtcDateTime,
}

pub async fn create_character<'c>(
    executor: impl DbExecutor<'c>,
    user_id: &UserId,
    name: &str,
    portrait: &str,
) -> Result<Option<UserCharacterId>, sqlx::Error> {
    let character_id = uuid::Uuid::new_v4();

    let res = sqlx::query!(
        r#"
        INSERT INTO characters (character_id, user_id, character_name, portrait)
        VALUES ($1, $2, $3, $4)
        "#,
        character_id,
        user_id,
        name,
        portrait
    )
    .execute(executor)
    .await;

    match res {
        Ok(_) => Ok(Some(character_id)),
        Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => Ok(None),
        Err(e) => Err(e),
    }
}

pub async fn get_character_by_name<'c>(
    executor: impl DbExecutor<'c>,
    character_name: &str,
) -> Result<Option<UserCharacterId>, sqlx::Error> {
    sqlx::query_scalar!(
        r#"
        SELECT 
            character_id as "character_id: UserCharacterId"
        FROM characters
        WHERE LOWER(character_name) = LOWER($1) AND deleted_at IS NULL
        "#,
        character_name
    )
    .fetch_optional(executor)
    .await
}

pub async fn read_character<'c>(
    executor: impl DbExecutor<'c>,
    character_id: &UserCharacterId,
) -> Result<Option<CharacterEntry>, sqlx::Error> {
    sqlx::query_as!(
        CharacterEntry,
        r#"
        SELECT
            characters.character_id as "character_id: UserCharacterId",
            user_id as "user_id: UserId",
            character_name,
            portrait,
            max_area_level as "max_area_level!: i32",
            resource_gems,
            resource_shards,
            resource_gold,
            created_at,
            updated_at,
            deleted_at as "deleted_at?: UtcDateTime",
            saved_game_instances.area_id as "area_id?",
            saved_game_instances.area_level as "area_level?: i32"
        FROM characters
        LEFT OUTER JOIN saved_game_instances
        ON characters.character_id = saved_game_instances.character_id
        WHERE characters.character_id = $1
        "#,
        character_id
    )
    .fetch_optional(executor)
    .await
}

pub async fn read_character_area_completed<'c>(
    executor: impl DbExecutor<'c>,
    character_id: &UserCharacterId,
    area_id: &str,
) -> Result<Option<CharacterAreaEntry>, sqlx::Error> {
    sqlx::query_as!(
        CharacterAreaEntry,
        r#"
        SELECT
            character_id as "character_id: UserCharacterId",
            area_id,
            max_area_level as "max_area_level!: i32",
            created_at,
            updated_at
         FROM character_area_completed 
         WHERE character_id = $1 AND area_id = $2
         "#,
        character_id,
        area_id
    )
    .fetch_optional(executor)
    .await
}

pub async fn read_character_areas_completed<'c>(
    executor: impl DbExecutor<'c>,
    character_id: &UserCharacterId,
) -> Result<Vec<CharacterAreaEntry>, sqlx::Error> {
    sqlx::query_as!(
        CharacterAreaEntry,
        r#"
        SELECT
            character_id as "character_id: UserCharacterId",
            area_id,
            max_area_level as "max_area_level!: i32",
            created_at,
            updated_at
         FROM character_area_completed WHERE character_id = $1
         "#,
        character_id
    )
    .fetch_all(executor)
    .await
}

pub async fn read_all_user_characters<'c>(
    executor: impl DbExecutor<'c>,
    user_id: &UserId,
) -> Result<Vec<CharacterEntry>, sqlx::Error> {
    sqlx::query_as!(
        CharacterEntry,
        r#"
        SELECT
            characters.character_id as "character_id: UserCharacterId",
            user_id as "user_id: UserId",
            character_name,
            portrait,
            max_area_level as "max_area_level!: i32",
            resource_gems,
            resource_shards,
            resource_gold,
            created_at,
            updated_at,
            deleted_at as "deleted_at: UtcDateTime",
            saved_game_instances.area_id as "area_id?",
            saved_game_instances.area_level as "area_level?: i32"
        FROM characters 
        LEFT OUTER JOIN saved_game_instances
        ON characters.character_id = saved_game_instances.character_id
        WHERE user_id = $1 AND deleted_at IS NULL
        ORDER BY created_at ASC
        "#,
        user_id
    )
    .fetch_all(executor)
    .await
}

pub async fn count_user_characters<'c>(
    executor: impl DbExecutor<'c>,
    user_id: &UserId,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar!(
        r#"
        SELECT
        COUNT(*) as "count!"
        FROM characters WHERE user_id = $1 AND deleted_at IS NULL
        "#,
        user_id
    )
    .fetch_one(executor)
    .await
}

/// Add/remove resources to character
pub async fn update_character_resources<'c>(
    executor: impl DbExecutor<'c>,
    character_id: &UserCharacterId,
    resource_gems: f64,
    resource_shards: f64,
    resource_gold: f64,
) -> Result<CharacterResources, sqlx::Error> {
    sqlx::query_as!(
        CharacterResources,
        r#"
        UPDATE characters
        SET 
            resource_gems =  resource_gems + $2,
            resource_shards = resource_shards + $3,
            resource_gold = resource_gold + $4,
            updated_at = CURRENT_TIMESTAMP 
        WHERE character_id = $1
        RETURNING resource_gems, resource_shards, resource_gold
        "#,
        character_id,
        resource_gems,
        resource_shards,
        resource_gold,
    )
    .fetch_one(executor)
    .await
}

pub async fn update_character_max_area_level<'c>(
    executor: &mut Transaction<'c, Database>,
    character_id: &UserCharacterId,
    max_area_level: i32,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        UPDATE characters
        SET 
            max_area_level = CASE WHEN max_area_level > $2 THEN max_area_level ELSE $2 END,
            updated_at = CURRENT_TIMESTAMP 
        WHERE character_id = $1
        "#,
        character_id,
        max_area_level,
    )
    .execute(&mut **executor)
    .await?;

    Ok(())
}

pub async fn update_character_area_progress<'c>(
    executor: &mut Transaction<'c, Database>,
    character_id: &UserCharacterId,
    area_id: &str,
    delta_area_level: i32,
) -> Result<(), sqlx::Error> {
    if delta_area_level > 0 {
        sqlx::query!(
        "INSERT INTO character_area_completed (character_id, area_id, max_area_level) VALUES ($1, $2, $3)
         ON CONFLICT(character_id, area_id) DO UPDATE 
         SET max_area_level = CASE
            WHEN EXCLUDED.max_area_level > character_area_completed.max_area_level
            THEN EXCLUDED.max_area_level
            ELSE character_area_completed.max_area_level
        END,
        updated_at = CASE
            WHEN EXCLUDED.max_area_level > character_area_completed.max_area_level
            THEN CURRENT_TIMESTAMP
            ELSE character_area_completed.updated_at
        END;",
        character_id,
        area_id,
        delta_area_level
    )
    .execute(&mut **executor)
    .await?;
    }

    Ok(())
}

pub async fn delete_character<'c>(
    executor: impl DbExecutor<'c>,
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
    .execute(executor)
    .await?;

    Ok(())
}
