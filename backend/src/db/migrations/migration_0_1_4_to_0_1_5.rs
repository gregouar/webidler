use sqlx::{types::JsonValue, FromRow, Transaction};

use shared::data::{
    stash::StashType,
    user::{UserCharacterId, UserId},
};

use crate::{
    app_state::MasterStore,
    db::{
        self,
        market::MarketId,
        pool::{Database, DbPool},
        utc_datetime::UtcDateTime,
    },
    game::systems::items_controller,
};

pub async fn migrate(db_pool: &DbPool, master_store: &MasterStore) -> anyhow::Result<()> {
    let mut tx = db_pool.begin().await?;

    create_user_stashes(&mut tx).await?;
    migrate_market_items(&mut tx, master_store).await?;

    tx.commit().await?;
    Ok(())
}

#[derive(Debug, FromRow)]
pub struct OldMarketEntry {
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

    pub deleted_at: Option<UtcDateTime>,
    pub deleted_by_id: Option<UserCharacterId>,
    pub deleted_by_name: Option<String>,
}

async fn create_user_stashes<'c>(executor: &mut Transaction<'c, Database>) -> anyhow::Result<()> {
    let users = sqlx::query!(
        r#"
        SELECT 
            DISTINCT(characters.user_id) as "user_id: UserId"
        FROM 
            market_old
        INNER JOIN
            characters ON characters.character_id = market_old.character_id
        WHERE
            market_old.deleted_at IS NULL
        "#
    )
    .fetch_all(&mut **executor)
    .await?;

    for user in users {
        db::stashes::create_stash(
            &mut **executor,
            user.user_id,
            StashType::Market,
            0,
            "Legacy stash",
        )
        .await?;
    }

    Ok(())
}

async fn migrate_market_items<'c>(
    mut executor: &mut Transaction<'c, Database>,
    master_store: &MasterStore,
) -> anyhow::Result<()> {
    let old_records = sqlx::query_as!(
        OldMarketEntry,
        r#"
        SELECT 
            market_old.market_id, 
            owner.character_id as "character_id: UserCharacterId", 
            owner.character_name,
            recipient_id as "recipient_id?: UserCharacterId", 
            recipient.character_name as "recipient_name?",
            rejected,
            price as "price: f64",
            item_level as "item_level!: i32",
            item_data as "item_data: JsonValue",
            market_old.created_at,
            market_old.updated_at,
            market_old.deleted_at as "deleted_at?: UtcDateTime",
            deleted_by as "deleted_by_id?: UserCharacterId", 
            buyer.character_name as "deleted_by_name?"
        FROM 
            market_old
        INNER JOIN
            characters AS owner ON owner.character_id = market_old.character_id
        LEFT JOIN
            characters AS recipient ON recipient.character_id = market_old.recipient_id
        LEFT JOIN
            characters AS buyer ON buyer.character_id = market_old.deleted_by
        WHERE
            market_old.deleted_at IS NULL
        "#
    )
    .fetch_all(&mut **executor)
    .await?;

    for record in old_records {
        let stash = db::stashes::get_character_stash_by_type(
            &mut **executor,
            &record.character_id,
            StashType::Market,
        )
        .await?
        .ok_or(anyhow::anyhow!("missing stash"))?;

        let item_specs = items_controller::init_item_specs_from_store(
            &master_store.items_store,
            serde_json::from_value(record.item_data)?,
        )
        .ok_or(anyhow::anyhow!("missing item"))?;

        let stash_item_id = db::stash_items::store_item(
            &mut *executor,
            &stash.stash_id,
            &record.character_id,
            &item_specs,
        )
        .await?;

        let market_id = db::market::sell_item(
            &mut executor,
            &stash_item_id,
            record.recipient_id,
            record.price,
            (&item_specs).try_into()?,
        )
        .await?;

        sqlx::query!(
            r#"
                UPDATE market SET
                    created_at = $1
                WHERE market_id = $2
                "#,
            record.updated_at,
            market_id
        )
        .execute(&mut **executor)
        .await?;
    }

    sqlx::query!("DELETE FROM market_old")
        .execute(&mut **executor)
        .await?;

    Ok(())
}
