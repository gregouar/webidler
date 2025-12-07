use std::collections::HashSet;

use sqlx::{types::JsonValue, FromRow, Transaction};

use shared::data::{
    item::{ItemSpecs, WeaponSpecs},
    item_affix::AffixEffectScope,
    market::MarketFilters,
    skill::DamageType,
    stash::StashId,
    user::{UserCharacterId, UserId},
};
use strum::IntoEnumIterator;

use crate::{
    constants::DATA_VERSION,
    db::{
        pool::{Database, DbExecutor},
        utc_datetime::UtcDateTime,
    },
};

pub type StashItemId = i64;

#[derive(Debug, FromRow)]
pub struct StashItemEntry {
    pub stash_id: StashId,
    pub stash_item_id: StashItemId,

    pub user_id: UserId,
    pub character_id: Option<UserCharacterId>,
    pub character_name: Option<String>,

    pub item_data: JsonValue,

    pub created_at: UtcDateTime,
}

pub struct StashItemFlattenStats {
    pub base_item_id: String,
    pub item_name: String,
    pub item_rarity: String,
    pub item_level: i32,
    pub item_armor: Option<f64>,
    pub item_block: Option<f64>,
    pub item_damages: Option<f64>,
    pub item_damage_physical: Option<f64>,
    pub item_damage_fire: Option<f64>,
    pub item_damage_poison: Option<f64>,
    pub item_damage_storm: Option<f64>,
    pub item_crit_chance: Option<f64>,
    pub item_crit_damage: Option<f64>,
}

impl TryFrom<&ItemSpecs> for StashItemFlattenStats {
    type Error = anyhow::Error;

    fn try_from(value: &ItemSpecs) -> Result<Self, Self::Error> {
        let item_damages = value.weapon_specs.as_ref().map(|weapon_specs| {
            DamageType::iter()
                .flat_map(|damage_type| average_weapon_damage(weapon_specs, damage_type))
                .sum()
        });
        let item_damage_physical = value
            .weapon_specs
            .as_ref()
            .and_then(|weapon_specs| average_weapon_damage(weapon_specs, DamageType::Physical));
        let item_damage_fire = value
            .weapon_specs
            .as_ref()
            .and_then(|weapon_specs| average_weapon_damage(weapon_specs, DamageType::Fire));
        let item_damage_poison = value
            .weapon_specs
            .as_ref()
            .and_then(|weapon_specs| average_weapon_damage(weapon_specs, DamageType::Poison));
        let item_damage_storm = value
            .weapon_specs
            .as_ref()
            .and_then(|weapon_specs| average_weapon_damage(weapon_specs, DamageType::Storm));

        Ok(Self {
            base_item_id: value.modifiers.base_item_id.clone(),
            item_name: value.base.name.clone(),
            item_rarity: serde_plain::to_string(&value.modifiers.rarity)?,
            item_level: value.required_level as i32,
            item_armor: value
                .armor_specs
                .as_ref()
                .map(|armor_specs| armor_specs.armor),
            item_block: value
                .armor_specs
                .as_ref()
                .map(|armor_specs| armor_specs.block as f64),
            item_damages,
            item_damage_physical,
            item_damage_fire,
            item_damage_poison,
            item_damage_storm,
            item_crit_chance: value
                .weapon_specs
                .as_ref()
                .map(|weapon_specs| weapon_specs.crit_chance.value as f64),
            item_crit_damage: value
                .weapon_specs
                .as_ref()
                .map(|weapon_specs| weapon_specs.crit_damage),
        })
    }
}

fn average_weapon_damage(weapon_specs: &WeaponSpecs, damage_type: DamageType) -> Option<f64> {
    weapon_specs
                .damage.get(&damage_type)
                .map(|value| 1.0 / (weapon_specs.cooldown as f64)
            // TODO: Lucky?
            * (1.0 + weapon_specs.crit_damage * weapon_specs.crit_chance.value as f64 * 0.0001)*(value.min + value.max) * 0.5)
}

pub async fn store_item<'c>(
    executor: &mut Transaction<'c, Database>,
    stash_id: &StashId,
    character_id: &UserCharacterId,
    item: &ItemSpecs,
) -> anyhow::Result<StashItemId> {
    Ok(create_stash_item(
        executor,
        stash_id,
        character_id,
        item.base
            .categories
            .iter()
            .filter_map(|category| serde_plain::to_string(&category).ok())
            .collect(),
        item.modifiers
            .aggregate_effects(AffixEffectScope::Global)
            .0
            .into_iter()
            .filter_map(|((stat_type, modifier), stat_value)| {
                Some((
                    serde_json::to_value(stat_type).ok()?,
                    serde_plain::to_string(&modifier).ok()?,
                    stat_value,
                ))
            })
            .collect(),
        item.try_into()?,
        serde_json::to_value(&item.modifiers)?,
    )
    .await?)
}

#[allow(clippy::too_many_arguments)]
async fn create_stash_item<'c>(
    executor: &mut Transaction<'c, Database>,
    stash_id: &StashId,
    character_id: &UserCharacterId,
    item_categories: HashSet<String>,
    item_stats: Vec<(JsonValue, String, f64)>,
    stash_item_flatten_stats: StashItemFlattenStats,
    item_data: JsonValue,
) -> Result<StashItemId, sqlx::Error> {
    let stash_item_id = sqlx::query_scalar!(
        r#"
        INSERT INTO stash_items (
            stash_id,
            character_id, 
            base_item_id, 
            item_name, 
            item_rarity, 
            item_level, 
            item_armor, 
            item_block, 
            item_damages, 
            item_damage_physical, 
            item_damage_fire, 
            item_damage_poison, 
            item_damage_storm, 
            item_crit_chance,
            item_crit_damage,
            item_data,
            data_version
        )
        VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17)
        RETURNING stash_item_id
        "#,
        stash_id,
        character_id,
        stash_item_flatten_stats.base_item_id,
        stash_item_flatten_stats.item_name,
        stash_item_flatten_stats.item_rarity,
        stash_item_flatten_stats.item_level,
        stash_item_flatten_stats.item_armor,
        stash_item_flatten_stats.item_block,
        stash_item_flatten_stats.item_damages,
        stash_item_flatten_stats.item_damage_physical,
        stash_item_flatten_stats.item_damage_fire,
        stash_item_flatten_stats.item_damage_poison,
        stash_item_flatten_stats.item_damage_storm,
        stash_item_flatten_stats.item_crit_chance,
        stash_item_flatten_stats.item_crit_damage,
        item_data,
        DATA_VERSION
    )
    .fetch_one(&mut **executor)
    .await?;

    for item_category in item_categories {
        sqlx::query!(
            "
        INSERT INTO stash_items_categories (stash_item_id, category)
        VALUES ($1,$2)
        ",
            stash_item_id,
            item_category,
        )
        .execute(&mut **executor)
        .await?;
    }

    for (item_stat, stat_modifier, stat_value) in item_stats {
        sqlx::query!(
            "
        INSERT INTO stash_items_stats (stash_item_id, item_stat, stat_modifier, stat_value)
        VALUES ($1,$2,$3,$4)
        ",
            stash_item_id,
            item_stat,
            stat_modifier,
            stat_value,
        )
        .execute(&mut **executor)
        .await?;
    }

    Ok(stash_item_id)
}

// pub async fn count_stash_items<'c>(
//     executor: impl DbExecutor<'c>,
//     stash_id: &StashId,
// ) -> anyhow::Result<i64> {
//     let row = sqlx::query!(
//         r#"
//         SELECT
//             COUNT(1) AS "count!"
//         FROM stash_items
//         WHERE deleted_at IS NULL
//           AND stash_id = $1
//         "#,
//         stash_id
//     )
//     .fetch_one(executor)
//     .await?;

//     Ok(row.count)
// }

pub async fn read_stash_item<'c>(
    executor: &mut Transaction<'c, Database>,
    stash_item_id: StashItemId,
) -> Result<Option<StashItemEntry>, sqlx::Error> {
    sqlx::query_as!(
        StashItemEntry,
        r#"
        SELECT 
            stash_items.stash_item_id,
            stash_items.stash_id as "stash_id: StashId",
            stashes.user_id as "user_id: UserId",
            characters.character_id as "character_id?: UserCharacterId", 
            characters.character_name as "character_name?: String",
            stash_items.item_data as "item_data: JsonValue",
            stash_items.created_at
        FROM 
            stash_items
        INNER JOIN
            stashes ON stashes.stash_id = stash_items.stash_id
        INNER JOIN
            characters ON characters.character_id = stash_items.character_id
        WHERE 
            stash_items.stash_item_id = $1
            AND stash_items.deleted_at is NULL            
        "#,
        stash_item_id
    )
    .fetch_optional(&mut **executor)
    .await
}

pub async fn read_stash_items<'c>(
    executor: impl DbExecutor<'c>,
    stash_id: StashId,
    filters: MarketFilters,
    skip: i64,
    limit: i64,
) -> anyhow::Result<(Vec<StashItemEntry>, bool)> {
    let limit_more = limit + 1;

    let no_filter_by_name = filters.item_name.is_none();
    let item_name = filters
        .item_name
        .map(|x| format!("%{}%", x.to_uppercase()))
        .unwrap_or_default();

    let item_level = filters.item_level.map(|x| x as i32).unwrap_or(i32::MAX);

    let no_filter_item_damages = filters.item_damages.is_none();
    let item_damages = filters.item_damages.unwrap_or_default();

    let no_filter_item_damage_physical = filters.item_damage_physical.is_none();
    let item_damage_physical = filters.item_damage_physical.unwrap_or_default();

    let no_filter_item_damage_fire = filters.item_damage_fire.is_none();
    let item_damage_fire = filters.item_damage_fire.unwrap_or_default();

    let no_filter_item_damage_poison = filters.item_damage_poison.is_none();
    let item_damage_poison = filters.item_damage_poison.unwrap_or_default();

    let no_filter_item_damage_storm = filters.item_damage_storm.is_none();
    let item_damage_storm = filters.item_damage_storm.unwrap_or_default();

    let no_filter_item_crit_chance = filters.item_crit_chance.is_none();
    let item_crit_chance = filters.item_crit_chance.unwrap_or_default();

    let no_filter_item_crit_damage = filters.item_crit_damage.is_none();
    let item_crit_damage = filters.item_crit_damage.unwrap_or_default();

    let no_filter_item_armor = filters.item_armor.is_none();
    let item_armor = filters.item_armor.unwrap_or_default();

    let no_filter_item_block = filters.item_block.is_none();
    let item_block = filters.item_block.unwrap_or_default();

    let item_rarity = filters
        .item_rarity
        .and_then(|x| serde_plain::to_string(&x).ok())
        .unwrap_or_default();

    let item_category = filters
        .item_category
        .and_then(|x| serde_plain::to_string(&x).ok())
        .unwrap_or_default();

    let order_by = serde_plain::to_string(&filters.order_by).unwrap_or_default();

    let stat_filters = filters.stat_filters.map(|stat_filter| {
        stat_filter
            .and_then(|stat_filter| {
                Some((
                    serde_json::to_value(stat_filter.stat).ok()?,
                    serde_plain::to_string(&stat_filter.modifier).ok()?,
                    stat_filter.value,
                ))
            })
            .unwrap_or_default()
    });

    let raw_items = sqlx::query_as!(
        StashItemEntry,
        r#"
        SELECT 
            stash_items.stash_id as "stash_id: StashId", 
            stash_items.stash_item_id, 
            stashes.user_id as "user_id: UserId",
            owner.character_id as "character_id?: UserCharacterId", 
            owner.character_name as "character_name?: String",
            item_data as "item_data: JsonValue",
            stash_items.created_at
        FROM 
            stash_items 
        INNER JOIN
            stashes ON stashes.stash_id = stash_items.stash_id
        INNER JOIN
            characters AS owner ON owner.character_id = stash_items.character_id
        LEFT JOIN
            stash_items_stats AS stat1 ON stat1.stash_item_id = stash_items.stash_item_id
                AND stat1.item_stat = $28
                AND stat1.stat_modifier = $29
        LEFT JOIN
            stash_items_stats AS stat2 ON stat2.stash_item_id = stash_items.stash_item_id
                AND stat2.item_stat = $31
                AND stat2.stat_modifier = $32
        LEFT JOIN
            stash_items_stats AS stat3 ON stat3.stash_item_id = stash_items.stash_item_id
                AND stat3.item_stat = $34
                AND stat3.stat_modifier = $35
        LEFT JOIN
            stash_items_stats AS stat4 ON stat4.stash_item_id = stash_items.stash_item_id
                AND stat4.item_stat = $37
                AND stat4.stat_modifier = $38
        LEFT JOIN
            stash_items_stats AS stat5 ON stat5.stash_item_id = stash_items.stash_item_id
                AND stat5.item_stat = $40
                AND stat5.stat_modifier = $41
        WHERE 
            stash_items.stash_id = $4
            AND stash_items.deleted_at IS NULL
            AND ($5 OR UPPER(stash_items.item_name) LIKE $6)
            AND (stash_items.item_level <= $7)
            AND ($8 = '' OR stash_items.item_rarity = $8)
            AND ($9 = '' OR EXISTS (
                SELECT 1
                FROM stash_items_categories cat
                WHERE cat.stash_item_id = stash_items.stash_item_id
                AND cat.category = $9
            ))
            AND ($10 OR stash_items.item_damages >= $11)
            AND ($12 OR stash_items.item_damage_physical >= $13)
            AND ($14 OR stash_items.item_damage_fire >= $15)
            AND ($16 OR stash_items.item_damage_poison >= $17)
            AND ($18 OR stash_items.item_damage_storm >= $19)
            AND ($20 OR stash_items.item_crit_chance >= $21)
            AND ($22 OR stash_items.item_crit_damage >= $23)
            AND ($24 OR stash_items.item_armor >= $25)
            AND ($26 OR stash_items.item_block >= $27)
            AND ($29 = '' OR stat1.stat_value >= $30)
            AND ($32 = '' OR stat2.stat_value >= $33)
            AND ($35 = '' OR stat3.stat_value >= $36)
            AND ($38 = '' OR stat4.stat_value >= $39)
            AND ($41 = '' OR stat5.stat_value >= $42)
        ORDER BY 
            CASE
                WHEN  $3 = 'Level' THEN stash_items.item_level
            END ASC,
            CASE
                WHEN  $3 = 'Damage' THEN  stash_items.item_damages
                WHEN  $3 = 'DamagePhysical' THEN  stash_items.item_damage_physical
                WHEN  $3 = 'DamageFire' THEN  stash_items.item_damage_fire
                WHEN  $3 = 'DamagePoison' THEN  stash_items.item_damage_poison
                WHEN  $3 = 'DamageStorm' THEN  stash_items.item_damage_storm
                WHEN  $3 = 'CritChance' THEN  stash_items.item_crit_chance
                WHEN  $3 = 'CritDamage' THEN  stash_items.item_crit_damage
                WHEN  $3 = 'Armor' THEN  stash_items.item_armor
                WHEN  $3 = 'Block' THEN  stash_items.item_block
            END DESC NULLS LAST, 
            CASE WHEN $3 = 'StatFilters' THEN stat1.stat_value END DESC NULLS LAST,
            CASE WHEN $3 = 'StatFilters' THEN stat2.stat_value END DESC NULLS LAST,
            CASE WHEN $3 = 'StatFilters' THEN stat3.stat_value END DESC NULLS LAST,
            CASE WHEN $3 = 'StatFilters' THEN stat4.stat_value END DESC NULLS LAST,
            CASE WHEN $3 = 'StatFilters' THEN stat5.stat_value END DESC NULLS LAST,
            stash_items.created_at DESC
        LIMIT $1
        OFFSET $2
        "#,
        limit_more, // $1
        skip,       // $2
        order_by,   // $3
        stash_id,
        no_filter_by_name, // $5
        item_name,
        item_level,
        item_rarity,
        item_category,
        no_filter_item_damages, // $10
        item_damages,
        no_filter_item_damage_physical,
        item_damage_physical,
        no_filter_item_damage_fire,
        item_damage_fire, // $15
        no_filter_item_damage_poison,
        item_damage_poison,
        no_filter_item_damage_storm,
        item_damage_storm,
        no_filter_item_crit_chance, // 20
        item_crit_chance,
        no_filter_item_crit_damage,
        item_crit_damage,
        no_filter_item_armor,
        item_armor, // $25
        no_filter_item_block,
        item_block,
        stat_filters[0].0,
        stat_filters[0].1,
        stat_filters[0].2, // 30
        stat_filters[1].0,
        stat_filters[1].1,
        stat_filters[1].2,
        stat_filters[2].0,
        stat_filters[2].1, // $35
        stat_filters[2].2,
        stat_filters[3].0,
        stat_filters[3].1,
        stat_filters[3].2,
        stat_filters[4].0, // $40
        stat_filters[4].1,
        stat_filters[4].2,
    )
    .fetch_all(executor)
    .await?;

    let has_more = raw_items.len() as i64 == limit_more;

    Ok((
        raw_items.into_iter().take(limit as usize).collect(),
        has_more,
    ))
}

pub async fn take_item<'c>(
    executor: &mut Transaction<'c, Database>,
    stash_id: Option<StashId>,
    stash_item_id: StashItemId,
) -> Result<Option<StashItemEntry>, sqlx::Error> {
    sqlx::query!(
        "UPDATE stash_items_categories SET deleted_at = CURRENT_TIMESTAMP WHERE stash_item_id = $1",
        stash_item_id
    )
    .execute(&mut **executor)
    .await?;

    sqlx::query!(
        "UPDATE stash_items_stats SET deleted_at = CURRENT_TIMESTAMP WHERE stash_item_id = $1",
        stash_item_id
    )
    .execute(&mut **executor)
    .await?;

    let skip_verify_stash_id = stash_id.is_none();

    sqlx::query_as!(
        StashItemEntry,
        r#"
        UPDATE 
            stash_items
        SET 
            deleted_at = CURRENT_TIMESTAMP
        WHERE 
            stash_item_id = $1
            AND ($2 OR stash_id = $3)
            AND deleted_at is NULL
        RETURNING
            stash_id as "stash_id: StashId",
            (
                SELECT stashes.user_id
                FROM stashes
                WHERE stashes.stash_id = stash_items.stash_id
            ) as "user_id: UserId",
            stash_item_id, 
            character_id as "character_id?: UserCharacterId", 
            NULL as "character_name?: String",
            item_data as "item_data: JsonValue",
            created_at
        "#,
        stash_item_id,
        skip_verify_stash_id,
        stash_id
    )
    .fetch_optional(&mut **executor)
    .await
}
