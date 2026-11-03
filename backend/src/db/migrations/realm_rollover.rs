use std::{cmp::Ordering, collections::HashMap};

use anyhow::{Context, ensure};
use shared::data::{
    stash::{StashId, StashType},
    user::UserCharacterId,
};
use sqlx::{Transaction, types::Json};

use crate::db::{pool::Database, utc_datetime::UtcDateTime};

const LEADERBOARD_SIZE: usize = 10;

pub async fn rollover_realms(
    executor: &mut Transaction<'static, Database>,
    realm_migrations: &[(&str, &str)],
    target_data_version: &str,
) -> anyhow::Result<()> {
    for &(source_realm, target_realm) in realm_migrations {
        ensure!(
            source_realm != target_realm,
            "source and target realms must differ"
        );

        migrate_stashes(executor, source_realm, target_realm, target_data_version)
            .await
            .with_context(|| format!("migrate stashes from {source_realm} to {target_realm}"))?;
        migrate_leaderboard(executor, source_realm, target_realm, target_data_version)
            .await
            .with_context(|| {
                format!("migrate leaderboard from {source_realm} to {target_realm}")
            })?;
        migrate_realm_rows(executor, source_realm, target_realm, target_data_version)
            .await
            .with_context(|| format!("migrate rows from {source_realm} to {target_realm}"))?;
    }

    Ok(())
}

async fn migrate_leaderboard(
    executor: &mut Transaction<'static, Database>,
    source_realm: &str,
    target_realm: &str,
    target_data_version: &str,
) -> anyhow::Result<()> {
    let source_entries = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) as "count!"
        FROM leaderboard
        WHERE realm_id = $1
          AND data_version < $2
        "#,
        source_realm,
        target_data_version,
    )
    .fetch_one(&mut **executor)
    .await?;
    if source_entries == 0 {
        return Ok(());
    }

    let mut entries = sqlx::query_as!(
        LeaderboardMigrationEntry,
        r#"
        DELETE
        FROM leaderboard
        WHERE realm_id IN ($1, $2)
        RETURNING
            character_id as "character_id: UserCharacterId",
            area_id,
            area_level as "area_level: i32",
            elapsed_time,
            created_at as "created_at: UtcDateTime",
            updated_at as "updated_at: UtcDateTime"
        "#,
        target_realm,
        source_realm,
    )
    .fetch_all(&mut **executor)
    .await?;

    entries.sort_by(|left, right| {
        left.area_id
            .cmp(&right.area_id)
            .then_with(|| compare_leaderboard_entries(left, right))
    });

    let mut current_area = None;
    let mut area_entries = 0;
    entries.retain(|entry| {
        if current_area.as_deref() != Some(entry.area_id.as_str()) {
            current_area = Some(entry.area_id.clone());
            area_entries = 0;
        }

        area_entries += 1;
        area_entries <= LEADERBOARD_SIZE
    });

    for entry in entries {
        #[cfg(feature = "sqlite")]
        let created_at = entry.created_at;
        #[cfg(feature = "postgres")]
        let created_at: chrono::DateTime<chrono::Utc> = entry.created_at.into();

        #[cfg(feature = "sqlite")]
        let updated_at = entry.updated_at;
        #[cfg(feature = "postgres")]
        let updated_at: chrono::DateTime<chrono::Utc> = entry.updated_at.into();

        sqlx::query!(
            r#"
            INSERT INTO leaderboard (
                character_id,
                realm_id,
                area_id,
                area_level,
                elapsed_time,
                data_version,
                created_at,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
            entry.character_id,
            target_realm,
            entry.area_id,
            entry.area_level,
            entry.elapsed_time,
            target_data_version,
            created_at,
            updated_at,
        )
        .execute(&mut **executor)
        .await?;
    }

    Ok(())
}

#[derive(Debug)]
struct LeaderboardMigrationEntry {
    character_id: UserCharacterId,
    area_id: String,
    area_level: i32,
    elapsed_time: f64,
    created_at: UtcDateTime,
    updated_at: UtcDateTime,
}

fn compare_leaderboard_entries(
    left: &LeaderboardMigrationEntry,
    right: &LeaderboardMigrationEntry,
) -> Ordering {
    right
        .area_level
        .cmp(&left.area_level)
        .then_with(|| left.elapsed_time.total_cmp(&right.elapsed_time))
        .then_with(|| {
            let left_updated: chrono::DateTime<chrono::Utc> = left.updated_at.clone().into();
            let right_updated: chrono::DateTime<chrono::Utc> = right.updated_at.clone().into();
            left_updated.cmp(&right_updated)
        })
}

async fn migrate_realm_rows(
    executor: &mut Transaction<'static, Database>,
    source_realm: &str,
    target_realm: &str,
    target_data_version: &str,
) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
        UPDATE characters
        SET realm_id = $1
        WHERE realm_id = $2
          AND character_id IN (
              SELECT character_id
              FROM characters_data
              WHERE data_version < $3
          )
        "#,
        target_realm,
        source_realm,
        target_data_version,
    )
    .execute(&mut **executor)
    .await?;

    sqlx::query!(
        "UPDATE game_stats SET realm_id = $1 WHERE realm_id = $2 AND data_version < $3",
        target_realm,
        source_realm,
        target_data_version,
    )
    .execute(&mut **executor)
    .await?;

    sqlx::query!(
        r#"
        UPDATE market
        SET realm_id = $1
        WHERE realm_id = $2
          AND stash_item_id IN (
              SELECT stash_item_id
              FROM stash_items
              WHERE data_version < $3
          )
        "#,
        target_realm,
        source_realm,
        target_data_version,
    )
    .execute(&mut **executor)
    .await?;

    sqlx::query!(
        r#"
        UPDATE stash_items_categories
        SET realm_id = $1
        WHERE realm_id = $2
          AND stash_item_id IN (
              SELECT stash_item_id
              FROM stash_items
              WHERE data_version < $3
          )
        "#,
        target_realm,
        source_realm,
        target_data_version,
    )
    .execute(&mut **executor)
    .await?;

    sqlx::query!(
        r#"
        UPDATE stash_items_stats
        SET realm_id = $1
        WHERE realm_id = $2
          AND stash_item_id IN (
              SELECT stash_item_id
              FROM stash_items
              WHERE data_version < $3
          )
        "#,
        target_realm,
        source_realm,
        target_data_version,
    )
    .execute(&mut **executor)
    .await?;

    Ok(())
}

async fn migrate_stashes(
    executor: &mut Transaction<'static, Database>,
    source_realm: &str,
    target_realm: &str,
    target_data_version: &str,
) -> anyhow::Result<()> {
    let source_stashes = sqlx::query_as!(
        StashMigrationEntry,
        r#"
        SELECT
            stash_id as "stash_id: StashId",
            owner_id as "owner_id: uuid::Uuid",
            stash_type as "stash_type: Json<StashType>",
            resource_gems,
            max_items
        FROM stashes
        WHERE realm_id = $1
          AND data_version < $2
          AND deleted_at IS NULL
        "#,
        source_realm,
        target_data_version,
    )
    .fetch_all(&mut **executor)
    .await?;

    let mut target_stashes = sqlx::query_as!(
        StashMigrationEntry,
        r#"
        SELECT
            stash_id as "stash_id: StashId",
            owner_id as "owner_id: uuid::Uuid",
            stash_type as "stash_type: Json<StashType>",
            resource_gems,
            max_items
        FROM stashes
        WHERE realm_id = $1
          AND deleted_at IS NULL
        "#,
        target_realm,
    )
    .fetch_all(&mut **executor)
    .await?;

    let mut target_indices = target_stashes
        .iter()
        .enumerate()
        .map(|(index, stash)| ((stash.owner_id, stash.stash_type.0), index))
        .collect::<HashMap<_, _>>();

    for source in source_stashes {
        let stash_key = (source.owner_id, source.stash_type.0);
        if let Some(target_index) = target_indices.get(&stash_key).copied() {
            let target = &mut target_stashes[target_index];
            sqlx::query!(
                "UPDATE stash_items SET stash_id = $1 WHERE stash_id = $2",
                target.stash_id,
                source.stash_id,
            )
            .execute(&mut **executor)
            .await?;

            target.resource_gems += source.resource_gems;
            target.max_items = target.max_items.max(source.max_items);
            sqlx::query!(
                r#"
                UPDATE stashes
                SET resource_gems = $2,
                    max_items = $3,
                    data_version = $4,
                    updated_at = CURRENT_TIMESTAMP
                WHERE stash_id = $1
                "#,
                target.stash_id,
                target.resource_gems,
                target.max_items,
                target_data_version,
            )
            .execute(&mut **executor)
            .await?;

            sqlx::query!("DELETE FROM stashes WHERE stash_id = $1", source.stash_id)
                .execute(&mut **executor)
                .await?;
        } else {
            sqlx::query!(
                r#"
                UPDATE stashes
                SET realm_id = $2,
                    data_version = $3,
                    updated_at = CURRENT_TIMESTAMP
                WHERE stash_id = $1
                "#,
                source.stash_id,
                target_realm,
                target_data_version,
            )
            .execute(&mut **executor)
            .await?;

            target_indices.insert(stash_key, target_stashes.len());
            target_stashes.push(source);
        }
    }

    Ok(())
}

#[derive(Debug)]
struct StashMigrationEntry {
    stash_id: StashId,
    owner_id: uuid::Uuid,
    stash_type: Json<StashType>,
    resource_gems: f64,
    max_items: i64,
}

pub async fn finalize_data_versions(
    executor: &mut Transaction<'static, Database>,
    target_data_version: &str,
) -> anyhow::Result<()> {
    sqlx::query!(
        "UPDATE characters_data SET data_version = $1 WHERE data_version < $1",
        target_data_version,
    )
    .execute(&mut **executor)
    .await?;
    sqlx::query!(
        "UPDATE stash_items SET data_version = $1 WHERE data_version < $1",
        target_data_version,
    )
    .execute(&mut **executor)
    .await?;
    sqlx::query!(
        "UPDATE stashes SET data_version = $1 WHERE data_version < $1",
        target_data_version,
    )
    .execute(&mut **executor)
    .await?;
    sqlx::query!(
        "UPDATE game_stats SET data_version = $1 WHERE data_version < $1",
        target_data_version,
    )
    .execute(&mut **executor)
    .await?;
    sqlx::query!(
        "UPDATE leaderboard SET data_version = $1 WHERE data_version < $1",
        target_data_version,
    )
    .execute(&mut **executor)
    .await?;

    Ok(())
}
