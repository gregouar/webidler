use std::collections::HashSet;

use sqlx::{FromRow, Transaction};

use shared::data::{
    area::AreaLevel,
    item::{ItemModifiers, ItemSpecs},
    item_affix::AffixEffectScope,
    user::UserCharacterId,
};

use crate::db::{
    pool::{Database, DbExecutor, DbPool},
    utc_datetime::UtcDateTime,
};

pub type MarketId = i64;

#[derive(Debug, FromRow)]
struct MarketEntry {
    pub market_id: MarketId,

    pub character_id: UserCharacterId, // TODO: replace or add character name
    pub private_sale: Option<UserCharacterId>, // For private offers
    pub rejected: bool,

    pub price: f64,

    pub item_level: AreaLevel,
    pub item_data: Vec<u8>,

    pub created_at: UtcDateTime,
    pub updated_at: UtcDateTime,
}

pub struct MarketItemEntry {
    pub item_id: usize,
    pub price: f64,

    pub character_id: UserCharacterId, // TODO: replace or add character name
    pub private_sale: Option<UserCharacterId>, // For private offers
    pub rejected: bool,

    pub item_level: AreaLevel,
    pub item_modifiers: ItemModifiers,

    pub created_at: UtcDateTime,
    pub updated_at: UtcDateTime,
}

pub async fn sell_item<'c>(
    executor: &mut Transaction<'c, Database>,
    character_id: &UserCharacterId,
    private_sale: Option<UserCharacterId>,
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
        private_sale,
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
                    serde_json::to_vec(&stat_type).ok()?,
                    serde_plain::to_string(&modifier).ok()?,
                    stat_value,
                ))
            })
            .collect(),
        item.modifiers.base_item_id.clone(),
        item.base.name.clone(),
        serde_plain::to_string(&item.modifiers.rarity)?,
        item.modifiers.level,
        item.armor_specs
            .as_ref()
            .map(|armor_specs| armor_specs.armor),
        item.armor_specs
            .as_ref()
            .map(|armor_specs| armor_specs.block as f64),
        item_damages,
        serde_json::to_vec(&item.modifiers)?,
    )
    .await?)
}

async fn create_market_item<'c>(
    executor: &mut Transaction<'c, Database>,
    character_id: &UserCharacterId,
    private_sale: Option<UserCharacterId>,
    price: f64,
    item_categories: HashSet<String>,
    item_stats: Vec<(Vec<u8>, String, f64)>,
    base_item_id: String,
    item_name: String,
    item_rarity: String,
    item_level: AreaLevel,
    item_armor: Option<f64>,
    item_block: Option<f64>,
    item_damages: Option<f64>,
    item_data: Vec<u8>,
) -> Result<(), sqlx::Error> {
    let market_id = sqlx::query_scalar!(
        "
        INSERT INTO market (
            character_id, 
            private_sale, 
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
        ",
        character_id,
        private_sale,
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

// TODO: filters
pub async fn load_market_items(
    executor: &DbPool,
    character_id: &UserCharacterId,
    own_listings: bool,
    skip: i64,
    limit: i64,
) -> anyhow::Result<(Vec<MarketItemEntry>, bool)> {
    let limit_more = limit + 1;
    let raw_items = sqlx::query_as!(
        MarketEntry,
        "
        SELECT 
            market_id as 'market_id: MarketId', 
            character_id as 'character_id: UserCharacterId', 
            private_sale as 'private_sale?: UserCharacterId', 
            rejected,
            price as 'price: f64',
            item_level as 'item_level: AreaLevel',
            item_data,
            created_at,
            updated_at
        FROM 
            market 
        WHERE 
            deleted_at IS NULL 
            AND (
                (NOT $4 
                    AND character_id != $3 
                    AND (private_sale = $3 OR private_sale IS NULL)
                    AND NOT rejected
                )
                OR ($4 
                    AND character_id = $3
                )
            )
        ORDER BY 
            rejected DESC, 
            private_sale DESC, 
            price ASC
        LIMIT $1
        OFFSET $2
        ",
        limit_more,
        skip,
        character_id,
        own_listings,
    )
    .fetch_all(executor)
    .await?;

    let has_more = raw_items.len() as i64 == limit_more;

    Ok((
        raw_items
            .into_iter()
            .take(limit as usize)
            .filter_map(|market_entry| {
                Some(MarketItemEntry {
                    item_id: market_entry.market_id as usize,
                    price: market_entry.price,
                    character_id: market_entry.character_id,
                    private_sale: market_entry.private_sale,
                    rejected: market_entry.rejected,
                    item_modifiers: serde_json::from_slice(&market_entry.item_data).ok()?,
                    item_level: market_entry.item_level,
                    created_at: market_entry.created_at,
                    updated_at: market_entry.updated_at,
                })
            })
            .collect(),
        has_more,
    ))
}

// TODO: update (reject + price)
// WARNING: to avoid cheat, edit price should create a new entry and delete old one (and error if old one deleted before)

pub async fn reject_item<'c>(
    executor: impl DbExecutor<'c>,
    market_id: MarketId,
    character_id: &UserCharacterId,
) -> anyhow::Result<bool> {
    Ok(sqlx::query_scalar!(
        "
        UPDATE
            market
        SET
            rejected = 1,
            updated_at = CURRENT_TIMESTAMP
        WHERE
            market_id = $1 
            AND private_sale = $2
            AND deleted_at IS NULL
        RETURNING 
            market_id
        ",
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
) -> anyhow::Result<Option<MarketItemEntry>> {
    Ok(delete_market_item(executor, market_id)
        .await?
        .and_then(|market_entry| {
            Some(MarketItemEntry {
                item_id: market_entry.market_id as usize,
                price: market_entry.price,
                character_id: market_entry.character_id,
                private_sale: market_entry.private_sale,
                rejected: market_entry.rejected,
                item_modifiers: serde_json::from_slice(&market_entry.item_data).ok()?,
                item_level: market_entry.item_level,
                created_at: market_entry.created_at,
                updated_at: market_entry.updated_at,
            })
        }))
}

async fn delete_market_item<'c>(
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
        "
        UPDATE 
            market
        SET 
            deleted_at = CURRENT_TIMESTAMP
        WHERE 
            market_id = $1
            AND deleted_at is NULL
        RETURNING
            market_id as 'market_id: MarketId', 
            character_id as 'character_id: UserCharacterId', 
            private_sale as 'private_sale?: UserCharacterId', 
            rejected,
            price as 'price: f64',
            item_level as 'item_level: AreaLevel',
            item_data,
            created_at,
            updated_at
        ",
        market_id
    )
    .fetch_optional(&mut **executor)
    .await
}
