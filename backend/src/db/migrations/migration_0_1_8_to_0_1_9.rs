use sqlx::{Transaction, types::JsonValue};

use shared::data::{
    item::ItemModifiers,
    modifier::Modifier,
    stat_effect::{StatEffect, StatType},
    user::UserCharacterId,
};

use crate::{
    app_state::MasterStore,
    constants::DATA_VERSION,
    db::{
        self,
        characters_data::CharacterDataEntry,
        pool::{Database, DbPool},
    },
    game::data::inventory_data::InventoryData,
};

pub async fn migrate(db_pool: &DbPool, master_store: &MasterStore) -> anyhow::Result<()> {
    let mut tx = db_pool.begin().await?;

    migrate_game_instances(&mut tx, master_store).await?;
    migrate_player_data(&mut tx).await?;
    migrate_stash_items(&mut tx).await?;

    tx.commit().await?;
    Ok(())
}

async fn migrate_game_instances(
    executor: &mut Transaction<'static, Database>,
    master_store: &MasterStore,
) -> anyhow::Result<()> {
    let game_instances = sqlx::query!(
        r#"
        SELECT
            character_id as "character_id: UserCharacterId"
         FROM saved_game_instances
         WHERE data_version <= '0.1.8'
         "#,
    )
    .fetch_all(&mut **executor)
    .await?;

    for instance in game_instances.iter() {
        if let Some((mut game_data, _)) = db::game_instances::load_game_instance_data(
            &mut **executor,
            master_store,
            &instance.character_id,
        )
        .await?
        {
            for item in game_data.player_inventory.mutate().all_items_mut() {
                fix_bouffon_shield(&mut item.modifiers);
            }

            db::game_instances::save_game_instance_data(
                &mut **executor,
                &instance.character_id,
                game_data,
            )
            .await?;
        }
    }

    Ok(())
}

async fn migrate_player_data(executor: &mut Transaction<'static, Database>) -> anyhow::Result<()> {
    let characters_data = sqlx::query_as!(
        CharacterDataEntry,
        r#"
        SELECT
            character_id as "character_id: UserCharacterId",
            data_version,
            inventory_data,
            passives_data,
            benedictions_data,
            created_at,
            updated_at
         FROM characters_data
         WHERE data_version <= '0.1.8'
         "#,
    )
    .fetch_all(&mut **executor)
    .await?;

    for character_data in characters_data {
        let mut inventory_data: InventoryData =
            rmp_serde::from_slice(&character_data.inventory_data)?;

        for item_modifiers in inventory_data
            .bag
            .iter_mut()
            .chain(inventory_data.equipped.values_mut())
        {
            fix_bouffon_shield(item_modifiers);
        }

        db::characters_data::upsert_character_inventory_data(
            &mut **executor,
            &character_data.character_id,
            rmp_serde::to_vec(&inventory_data)?,
        )
        .await?;
    }

    Ok(())
}

async fn migrate_stash_items(executor: &mut Transaction<'static, Database>) -> anyhow::Result<()> {
    let market_entries = sqlx::query!(
        r#"
        SELECT
            stash_item_id,
            item_data as "item_data: JsonValue"
        FROM stash_items
        WHERE data_version <= '0.1.8'
        "#
    )
    .fetch_all(&mut **executor)
    .await?;

    let new_market_entries = market_entries
        .into_iter()
        .map(|market_entry| {
            let mut item_modifiers: ItemModifiers = serde_json::from_value(market_entry.item_data)?;
            fix_bouffon_shield(&mut item_modifiers);
            Ok((
                market_entry.stash_item_id,
                serde_json::to_value(&item_modifiers)?,
            ))
        })
        .collect::<anyhow::Result<Vec<_>>>()?;

    for (stash_item_id, item_data) in new_market_entries {
        sqlx::query!(
            "UPDATE stash_items SET item_data = $1, data_version = $2 WHERE stash_item_id = $3",
            item_data,
            DATA_VERSION,
            stash_item_id,
        )
        .execute(&mut **executor)
        .await?;
    }
    Ok(())
}

fn fix_bouffon_shield(item_modifiers: &mut ItemModifiers) {
    if item_modifiers.base_item_id == "heater_shield_unique" {
        for affix in item_modifiers.affixes.iter_mut() {
            for effect in affix.effects.iter_mut() {
                if let StatEffect {
                    stat: StatType::StatConditionalModifier { .. },
                    ..
                } = effect.stat_effect
                {
                    effect.stat_effect.modifier = Modifier::More;
                }
            }
        }
    }
}
