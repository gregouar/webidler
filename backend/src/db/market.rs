use std::collections::HashSet;

use sqlx::{FromRow, Transaction};

use shared::data::{area::AreaLevel, item::ItemSpecs, market::MarketItem, user::UserCharacterId};

use crate::db::{
    pool::{Database, DbExecutor},
    utc_datetime::UtcDateTime,
};

pub type ItemId = i64;

#[derive(Debug, FromRow)]
pub struct MarketEntry {
    pub item_id: ItemId,

    pub character_id: UserCharacterId, // TODO: replace or add character name
    pub private_sale: Option<UserCharacterId>, // For private offers
    pub rejected: bool,

    pub price: f64,

    pub item_data: Vec<u8>,

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
            * (1.0 + weapon_specs.crit_damage * weapon_specs.crit_chances as f64)
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
        item.name.clone(),
        serde_plain::to_string(&item.rarity)?,
        item.level,
        item.armor_specs
            .as_ref()
            .map(|armor_specs| armor_specs.armor),
        item.armor_specs
            .as_ref()
            .map(|armor_specs| armor_specs.block as f64),
        item_damages,
        serde_json::to_vec(item)?,
    )
    .await?)
}

async fn create_market_item<'c>(
    executor: &mut Transaction<'c, Database>,
    character_id: &UserCharacterId,
    private_sale: Option<UserCharacterId>,
    price: f64,
    item_categories: HashSet<String>,
    item_name: String,
    item_rarity: String,
    item_level: AreaLevel,
    item_armor: Option<f64>,
    item_block: Option<f64>,
    item_damages: Option<f64>,
    item_data: Vec<u8>,
) -> Result<(), sqlx::Error> {
    let item_id = sqlx::query_scalar!(
        "
        INSERT INTO market (character_id, private_sale, price, item_name, item_rarity, item_level, item_armor, item_block, item_damages, item_data)
        VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)
        RETURNING item_id
        ",
        character_id,
        private_sale,
        price,
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
        INSERT INTO market_categories (item_id, category)
        VALUES ($1,$2)
        ",
            item_id,
            item_category,
        )
        .execute(&mut **executor)
        .await?;
    }
    Ok(())
}

// TODO: filters
pub async fn load_market_items<'c>(
    executor: impl DbExecutor<'c>,
    // character_id: &UserCharacterId,
    skip: i64,
    limit: i64,
) -> anyhow::Result<Vec<MarketItem>> {
    Ok(read_market_items(executor, skip, limit)
        .await?
        .iter()
        .filter_map(|item_entry| {
            Some(MarketItem {
                item_id: item_entry.item_id as usize,
                item_specs: serde_json::from_slice(&item_entry.item_data).ok()?,
                price: item_entry.price,
            })
        })
        .collect())
}

async fn read_market_items<'c>(
    executor: impl DbExecutor<'c>,
    // character_id: &UserCharacterId,
    skip: i64,
    limit: i64,
) -> Result<Vec<MarketEntry>, sqlx::Error> {
    sqlx::query_as!(
        MarketEntry,
        "
        SELECT 
            item_id as 'item_id: ItemId', 
            character_id as 'character_id: UserCharacterId', 
            private_sale as 'private_sale?: UserCharacterId', 
            rejected,
            price as 'price: f64',
            item_data,
            created_at,
            updated_at
        FROM market 
        WHERE deleted_at IS NULL
        ORDER BY price ASC
        LIMIT $2
        OFFSET $1
        ",
        skip,
        limit,
    )
    .fetch_all(executor)
    .await
}
