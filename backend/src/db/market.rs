use std::collections::HashSet;

use sqlx::{types::JsonValue, FromRow, Transaction};

use shared::data::{
    item::ItemSpecs, item_affix::AffixEffectScope, market::MarketFilters, user::UserCharacterId,
};

use crate::db::{
    pool::{Database, DbExecutor},
    utc_datetime::UtcDateTime,
};

pub type MarketId = i64;

#[derive(Debug, FromRow)]
pub struct MarketEntry {
    pub market_id: MarketId,

    pub character_id: UserCharacterId,
    pub character_name: String,
    pub recipient_id: Option<UserCharacterId>, // For private offers
    pub recipient_name: Option<String>,        // For private offers
    pub rejected: bool,

    pub price: f64,

    pub item_level: i32,
    pub item_data: JsonValue,

    pub created_at: UtcDateTime,
    pub updated_at: UtcDateTime,
}

pub async fn sell_item<'c>(
    executor: &mut Transaction<'c, Database>,
    character_id: &UserCharacterId,
    recipient_id: Option<UserCharacterId>,
    price: f64,
    item: &ItemSpecs,
) -> anyhow::Result<()> {
    let item_damages = item.weapon_specs.as_ref().map(|weapon_specs| {
        1.0 / (weapon_specs.cooldown as f64)
            * (1.0 + weapon_specs.crit_damage * weapon_specs.crit_chances as f64 * 0.0001)
            * weapon_specs
                .damage
                .values()
                .map(|(min, max)| (min + max) * 0.5)
                .sum::<f64>()
    });

    Ok(create_market_item(
        executor,
        character_id,
        recipient_id,
        price,
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
        item.modifiers.base_item_id.clone(),
        item.base.name.clone(),
        serde_plain::to_string(&item.modifiers.rarity)?,
        item.modifiers.level as i32,
        item.armor_specs
            .as_ref()
            .map(|armor_specs| armor_specs.armor),
        item.armor_specs
            .as_ref()
            .map(|armor_specs| armor_specs.block as f64),
        item_damages,
        serde_json::to_value(&item.modifiers)?,
        // serde_json::to_vec(&item.modifiers)?.into(),
    )
    .await?)
}

#[allow(clippy::too_many_arguments)]
async fn create_market_item<'c>(
    executor: &mut Transaction<'c, Database>,
    character_id: &UserCharacterId,
    recipient_id: Option<UserCharacterId>,
    price: f64,
    item_categories: HashSet<String>,
    item_stats: Vec<(JsonValue, String, f64)>,
    base_item_id: String,
    item_name: String,
    item_rarity: String,
    item_level: i32,
    item_armor: Option<f64>,
    item_block: Option<f64>,
    item_damages: Option<f64>,
    item_data: JsonValue,
) -> Result<(), sqlx::Error> {
    let market_id = sqlx::query_scalar!(
        r#"
        INSERT INTO market (
            character_id, 
            recipient_id, 
            price, 
            base_item_id, 
            item_name, 
            item_rarity, 
            item_level, 
            item_armor, 
            item_block, 
            item_damages, 
            item_data
        )
        VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)
        RETURNING market_id
        "#,
        character_id,
        recipient_id,
        price,
        base_item_id,
        item_name,
        item_rarity,
        item_level,
        item_armor,
        item_block,
        item_damages,
        item_data,
    )
    .fetch_one(&mut **executor)
    .await?;

    for item_category in item_categories {
        sqlx::query!(
            "
        INSERT INTO market_categories (market_id, category)
        VALUES ($1,$2)
        ",
            market_id,
            item_category,
        )
        .execute(&mut **executor)
        .await?;
    }

    for (item_stat, stat_modifier, stat_value) in item_stats {
        sqlx::query!(
            "
        INSERT INTO market_stats (market_id, item_stat, stat_modifier, stat_value)
        VALUES ($1,$2,$3,$4)
        ",
            market_id,
            item_stat,
            stat_modifier,
            stat_value,
        )
        .execute(&mut **executor)
        .await?;
    }

    Ok(())
}

pub async fn count_market_items<'c>(
    executor: impl DbExecutor<'c>,
    character_id: &UserCharacterId,
) -> anyhow::Result<(i64, i64)> {
    let row = sqlx::query!(
        r#"
        SELECT
            COUNT(CASE WHEN recipient_id IS NULL THEN 1 END) AS "public_items!",
            COUNT(CASE WHEN recipient_id IS NOT NULL THEN 1 END) AS "private_items!"
        FROM market
        WHERE deleted_at IS NULL
          AND character_id = $1
        "#,
        character_id
    )
    .fetch_one(executor)
    .await?;

    Ok((row.public_items, row.private_items))
}

pub async fn read_market_items<'c>(
    executor: impl DbExecutor<'c>,
    character_id: &UserCharacterId,
    own_listings: bool,
    filters: MarketFilters,
    skip: i64,
    limit: i64,
) -> anyhow::Result<(Vec<MarketEntry>, bool)> {
    let limit_more = limit + 1;

    let no_filter_by_name = filters.item_name.is_none();
    let item_name = filters
        .item_name
        .map(|x| format!("%{}%", x.to_uppercase()))
        .unwrap_or_default();

    let item_level = filters.item_level.map(|x| x as i32).unwrap_or(i32::MAX);
    let price = filters.price.map(|x| x.into_inner()).unwrap_or(f64::MAX);

    let no_filter_item_damages = filters.item_damages.is_none();
    let item_damages = filters.item_damages.unwrap_or_default();

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
                    stat_filter.value
                        * match stat_filter.modifier {
                            shared::data::stat_effect::Modifier::Multiplier => 0.01,
                            shared::data::stat_effect::Modifier::Flat => 1.0,
                        },
                ))
            })
            .unwrap_or_default()
    });

    let raw_items = sqlx::query_as!(
        MarketEntry,
        r#"
        SELECT 
            market_id, 
            owner.character_id as "character_id: UserCharacterId", 
            owner.character_name,
            recipient_id as "recipient_id?: UserCharacterId", 
            recipient.character_name as "recipient_name?",
            rejected,
            price as "price: f64",
            item_level as "item_level!: i32",
            item_data as "item_data: JsonValue",
            market.created_at,
            market.updated_at
        FROM 
            market 
        INNER JOIN
            characters AS owner ON owner.character_id = market.character_id
        LEFT JOIN
            characters AS recipient ON recipient.character_id = market.recipient_id
        WHERE 
            market.deleted_at IS NULL 
            AND (
                (NOT $4 
                    AND market.character_id != $3 
                    AND (recipient_id = $3 OR recipient_id IS NULL)
                    AND NOT rejected
                )
                OR ($4 
                    AND market.character_id = $3
                )
            )
            AND ($5 OR UPPER(market.item_name) LIKE $6)
            AND (market.item_level <= $7)
            AND (market.price <= $8)
            AND ($9 = '' OR market.item_rarity = $9)
            AND ($10 = '' OR EXISTS (
                SELECT 1
                FROM market_categories mc
                WHERE mc.market_id = market.market_id
                AND mc.category = $10
            ))
            AND ($11 OR market.item_damages >= $12)
            AND ($13 OR market.item_armor >= $14)
            AND ($15 OR market.item_block >= $16)
            AND ($19 = '' OR EXISTS (
                SELECT 1
                FROM market_stats ms
                WHERE ms.market_id = market.market_id
                AND ms.item_stat = $18
                AND ms.stat_modifier = $19
                AND ms.stat_value >= $20
            ))
            AND ($22 = '' OR EXISTS (
                SELECT 1
                FROM market_stats ms
                WHERE ms.market_id = market.market_id
                AND ms.item_stat = $21
                AND ms.stat_modifier = $22
                AND ms.stat_value >= $23
            ))
            AND ($25 = '' OR EXISTS (
                SELECT 1
                FROM market_stats ms
                WHERE ms.market_id = market.market_id
                AND ms.item_stat = $24
                AND ms.stat_modifier = $25
                AND ms.stat_value >= $26
            ))
            AND ($28 = '' OR EXISTS (
                SELECT 1
                FROM market_stats ms
                WHERE ms.market_id = market.market_id
                AND ms.item_stat = $27
                AND ms.stat_modifier = $28
                AND ms.stat_value >= $29
            ))
            AND ($31 = '' OR EXISTS (
                SELECT 1
                FROM market_stats ms
                WHERE ms.market_id = market.market_id
                AND ms.item_stat = $30
                AND ms.stat_modifier = $31
                AND ms.stat_value >= $32
            ))
        ORDER BY 
            rejected DESC, 
            recipient_id ASC, 
            CASE
                WHEN  $17 = 'Level' THEN market.item_level
            END ASC,
            CASE
                WHEN  $17 = 'Damages' THEN  market.item_damages
                WHEN  $17 = 'Armor' THEN  market.item_armor
                WHEN  $17 = 'Block' THEN  market.item_block
            END DESC NULLS LAST, 
            market.price ASC
        LIMIT $1
        OFFSET $2
        "#,
        limit_more, // $1
        skip,       // $2
        character_id,
        own_listings,
        no_filter_by_name, // $5
        item_name,
        item_level,
        price,
        item_rarity,
        item_category, // $10
        no_filter_item_damages,
        item_damages,
        no_filter_item_armor,
        item_armor,
        no_filter_item_block, // $15
        item_block,
        order_by,
        stat_filters[0].0,
        stat_filters[0].1,
        stat_filters[0].2, // $20
        stat_filters[1].0,
        stat_filters[1].1,
        stat_filters[1].2,
        stat_filters[2].0,
        stat_filters[2].1, // $25
        stat_filters[2].2,
        stat_filters[3].0,
        stat_filters[3].1,
        stat_filters[3].2,
        stat_filters[4].0, // $30
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

pub async fn reject_item<'c>(
    executor: impl DbExecutor<'c>,
    market_id: MarketId,
    character_id: &UserCharacterId,
) -> anyhow::Result<bool> {
    Ok(sqlx::query_scalar!(
        r#"
        UPDATE
            market
        SET
            rejected = TRUE,
            updated_at = CURRENT_TIMESTAMP
        WHERE
            market_id = $1 
            AND recipient_id = $2
            AND deleted_at IS NULL
        RETURNING 
            market_id
        "#,
        market_id,
        character_id
    )
    .fetch_optional(executor)
    .await?
    .is_some())
}

pub async fn buy_item<'c>(
    executor: &mut Transaction<'c, Database>,
    market_id: MarketId,
) -> Result<Option<MarketEntry>, sqlx::Error> {
    sqlx::query!(
        "UPDATE market_categories SET deleted_at = CURRENT_TIMESTAMP WHERE market_id = $1",
        market_id
    )
    .execute(&mut **executor)
    .await?;

    sqlx::query!(
        "UPDATE market_stats SET deleted_at = CURRENT_TIMESTAMP WHERE market_id = $1",
        market_id
    )
    .execute(&mut **executor)
    .await?;

    sqlx::query_as!(
        MarketEntry,
        r#"
        UPDATE 
            market
        SET 
            deleted_at = CURRENT_TIMESTAMP
        WHERE 
            market_id = $1
            AND deleted_at is NULL
        RETURNING
            market_id, 
            character_id as "character_id: UserCharacterId", 
            'owner' as "character_name!: String",
            recipient_id as "recipient_id?: UserCharacterId", 
            NULL as "recipient_name?: String",
            rejected,
            price as "price: f64",
            item_level as "item_level!: i32",
            item_data as "item_data: JsonValue",
            created_at,
            updated_at
        "#,
        market_id
    )
    .fetch_optional(&mut **executor)
    .await
}
