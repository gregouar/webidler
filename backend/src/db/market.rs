use sqlx::{types::JsonValue, FromRow, Transaction};

use shared::data::{
    market::MarketFilters,
    user::{UserCharacterId, UserId},
};

use crate::db::{
    pool::{Database, DbExecutor},
    stash_items::{StashItemFlattenStats, StashItemId},
    utc_datetime::UtcDateTime,
};

pub type MarketId = i64;

#[derive(Debug, FromRow)]
pub struct MarketEntry {
    pub market_id: MarketId,
    pub stash_item_id: StashItemId,

    pub owner_id: UserId,
    pub owner_name: String,

    pub recipient_id: Option<UserId>,   // For private offers
    pub recipient_name: Option<String>, // For private offers
    pub rejected: bool,

    pub price: f64,

    pub item_data: JsonValue,

    pub created_at: UtcDateTime,
    // // pub updated_at: UtcDateTime,
    pub deleted_at: Option<UtcDateTime>,
    pub deleted_by_id: Option<UserCharacterId>,
    pub deleted_by_name: Option<String>,
}

#[derive(Debug, FromRow)]
pub struct MarketBuyEntry {
    pub market_id: MarketId,
    pub stash_item_id: StashItemId,

    pub recipient_id: Option<UserId>, // For private offers

    pub price: f64,
}

pub async fn sell_item<'c>(
    executor: &mut Transaction<'c, Database>,
    stash_item_id: &StashItemId,
    recipient_id: Option<UserId>,
    price: f64,
    stash_item_flatten_stats: StashItemFlattenStats,
) -> Result<MarketId, sqlx::Error> {
    Ok(sqlx::query_scalar!(
        r#"
        INSERT INTO market (
            stash_item_id,
            recipient_id,
            price,
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
            item_crit_damage
        )
        VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16)
        RETURNING market_id
        "#,
        stash_item_id,
        recipient_id,
        price,
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
    )
    .fetch_one(&mut **executor)
    .await?)
}

pub async fn read_market_items<'c>(
    executor: impl DbExecutor<'c>,
    user_id: &UserId,
    filters: MarketFilters,
    skip: i64,
    limit: i64,
    own_listings: bool,
    is_deleted: bool,
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
        MarketEntry,
        r#"
        SELECT 
            market.market_id,
            stash_items.stash_item_id, 
            owner.user_id as "owner_id!: UserCharacterId", 
            owner.username as "owner_name!: String",
            market.recipient_id as "recipient_id?: UserId",
            recipient.username as "recipient_name?: String",
            market.rejected,
            market.price,
            stash_items.item_data as "item_data: JsonValue",
            market.created_at,
            market.deleted_at as "deleted_at?: UtcDateTime",
            buyer.user_id as "deleted_by_id?: UserId",
            buyer.username as "deleted_by_name?: String"
        FROM 
            market 
        INNER JOIN
            stash_items ON stash_items.stash_item_id = market.stash_item_id
        INNER JOIN
            stashes ON stashes.stash_id = stash_items.stash_id
        INNER JOIN
            users AS owner ON owner.user_id = stashes.user_id
        LEFT JOIN
            users AS recipient ON recipient.user_id = market.recipient_id
        LEFT JOIN
            users AS buyer ON buyer.user_id = market.deleted_by
        LEFT JOIN
            stash_items_stats AS stat1 ON stat1.stash_item_id = market.stash_item_id
                AND stat1.item_stat = $28
                AND stat1.stat_modifier = $29
        LEFT JOIN
            stash_items_stats AS stat2 ON stat2.stash_item_id = market.stash_item_id
                AND stat2.item_stat = $31
                AND stat2.stat_modifier = $32
        LEFT JOIN
            stash_items_stats AS stat3 ON stat3.stash_item_id = market.stash_item_id
                AND stat3.item_stat = $34
                AND stat3.stat_modifier = $35
        LEFT JOIN
            stash_items_stats AS stat4 ON stat4.stash_item_id = market.stash_item_id
                AND stat4.item_stat = $37
                AND stat4.stat_modifier = $38
        LEFT JOIN
            stash_items_stats AS stat5 ON stat5.stash_item_id = market.stash_item_id
                AND stat5.item_stat = $40
                AND stat5.stat_modifier = $41
        WHERE 
            (
                (
                    $45
                    AND market.deleted_at IS NOT NULL
                    AND market.deleted_by != $4
                )
                OR 
                (
                    NOT $45 
                    AND market.deleted_at IS NULL
                )
            )
            AND (
                (
                    $44
                    AND owner.user_id = $4
                )
                OR
                (
                    NOT $44
                    AND (recipient_id IS NULL OR recipient_id = $4)
                    AND NOT rejected
                )
            )
            AND ($5 OR UPPER(market.item_name) LIKE $6)
            AND (market.item_level <= $7)
            AND ($8 = '' OR market.item_rarity = $8)
            AND ($9 = '' OR EXISTS (
                SELECT 1
                FROM stash_items_categories cat
                WHERE cat.stash_item_id = market.stash_item_id
                AND cat.category = $9
            ))
            AND ($10 OR market.item_damages >= $11)
            AND ($12 OR market.item_damage_physical >= $13)
            AND ($14 OR market.item_damage_fire >= $15)
            AND ($16 OR market.item_damage_poison >= $17)
            AND ($18 OR market.item_damage_storm >= $19)
            AND ($20 OR market.item_crit_chance >= $21)
            AND ($22 OR market.item_crit_damage >= $23)
            AND ($24 OR market.item_armor >= $25)
            AND ($26 OR market.item_block >= $27)
            AND ($29 = '' OR stat1.stat_value >= $30)
            AND ($32 = '' OR stat2.stat_value >= $33)
            AND ($35 = '' OR stat3.stat_value >= $36)
            AND ($38 = '' OR stat4.stat_value >= $39)
            AND ($41 = '' OR stat5.stat_value >= $42)
            AND (market.price <= $43)
        ORDER BY 
            COALESCE(market.recipient_id = $4, false) DESC, 
            CASE
                WHEN  $3 = 'Level' THEN market.item_level
            END ASC,
            CASE
                WHEN  $3 = 'Damage' THEN  market.item_damages
                WHEN  $3 = 'DamagePhysical' THEN  market.item_damage_physical
                WHEN  $3 = 'DamageFire' THEN  market.item_damage_fire
                WHEN  $3 = 'DamagePoison' THEN  market.item_damage_poison
                WHEN  $3 = 'DamageStorm' THEN  market.item_damage_storm
                WHEN  $3 = 'CritChance' THEN  market.item_crit_chance
                WHEN  $3 = 'CritDamage' THEN  market.item_crit_damage
                WHEN  $3 = 'Armor' THEN  market.item_armor
                WHEN  $3 = 'Block' THEN  market.item_block
            END DESC NULLS LAST, 
            CASE WHEN $3 = 'StatFilters' THEN stat1.stat_value END DESC NULLS LAST,
            CASE WHEN $3 = 'StatFilters' THEN stat2.stat_value END DESC NULLS LAST,
            CASE WHEN $3 = 'StatFilters' THEN stat3.stat_value END DESC NULLS LAST,
            CASE WHEN $3 = 'StatFilters' THEN stat4.stat_value END DESC NULLS LAST,
            CASE WHEN $3 = 'StatFilters' THEN stat5.stat_value END DESC NULLS LAST,
            CASE 
                WHEN $3 = 'Time' THEN market.created_at
            END DESC,
            CASE
                WHEN $3 != 'Time' THEN market.price 
            END ASC
        LIMIT $1
        OFFSET $2
        "#,
        limit_more, // $1
        skip,       // $2
        order_by,   // $3
        user_id,
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
        price,
        own_listings,
        is_deleted,
    )
    .fetch_all(executor)
    .await?;

    let has_more = raw_items.len() as i64 == limit_more;

    Ok((
        raw_items.into_iter().take(limit as usize).collect(),
        has_more,
    ))
}

// pub async fn read_market_stash<'c>(
//     executor: impl DbExecutor<'c>,
//     user_id: &UserId,
//     filters: MarketFilters,
//     skip: i64,
//     limit: i64,
// ) -> anyhow::Result<(Vec<MarketEntry>, bool)> {
//     unimplemented!()
// }

pub async fn reject_item<'c>(
    executor: impl DbExecutor<'c>,
    market_id: MarketId,
    user_id: &UserId,
) -> anyhow::Result<bool> {
    Ok(sqlx::query_scalar!(
        r#"
        UPDATE
            market
        SET
            rejected = TRUE
        WHERE
            market_id = $1 
            AND recipient_id = $2
            AND deleted_at IS NULL
        RETURNING 
            market_id
        "#,
        market_id,
        user_id
    )
    .fetch_optional(executor)
    .await?
    .is_some())
}

pub async fn buy_item<'c>(
    executor: &mut Transaction<'c, Database>,
    market_id: MarketId,
    buyer: Option<UserId>,
) -> Result<Option<MarketBuyEntry>, sqlx::Error> {
    sqlx::query_as!(
        MarketBuyEntry,
        r#"
        UPDATE market
        SET 
            deleted_at = CURRENT_TIMESTAMP,
            deleted_by = $2
        WHERE 
            market.market_id = $1
            AND market.deleted_at IS NULL
        RETURNING
            market.market_id, 
            market.stash_item_id,
            market.recipient_id as "recipient_id?: UserCharacterId", 
            market.price as "price: f64"
        "#,
        market_id,
        buyer
    )
    .fetch_optional(&mut **executor)
    .await
}
