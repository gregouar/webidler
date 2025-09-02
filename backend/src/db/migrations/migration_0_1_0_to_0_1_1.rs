use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use shared::data::{
    area::AreaLevel,
    item::{self, ArmorSpecs, ItemBase, ItemModifiers, ItemRarity, ItemSlot, WeaponSpecs},
    item_affix::ItemAffix,
    player,
    trigger::TriggerSpecs,
    user::UserCharacterId,
};

use crate::{
    app_state::MasterStore,
    db::{
        self,
        pool::{DbExecutor, DbPool},
        utc_datetime::UtcDateTime,
    },
    game::data::items_store::ItemsStore,
};

#[derive(Debug, FromRow)]
#[allow(dead_code)]
struct CharacterDataEntry {
    pub character_id: UserCharacterId,

    pub data_version: String,
    pub inventory_data: Vec<u8>,
    pub passives_data: Option<Vec<u8>>,

    pub created_at: UtcDateTime,
    pub updated_at: UtcDateTime,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OldItemSpecs {
    pub name: String,

    pub base: ItemBase,
    pub rarity: ItemRarity,
    pub level: AreaLevel,

    pub weapon_specs: Option<WeaponSpecs>,
    pub armor_specs: Option<ArmorSpecs>,

    pub affixes: Vec<ItemAffix>,
    pub triggers: Vec<TriggerSpecs>,

    #[serde(default)] // TODO: Remove later, only for save backward comp
    pub old_game: bool, // To indicate it comes from old game and not dropped during current one
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct OldPlayerInventory {
    pub equipped: HashMap<ItemSlot, OldEquippedSlot>,

    pub bag: Vec<OldItemSpecs>,
    pub max_bag_size: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum OldEquippedSlot {
    MainSlot(Box<OldItemSpecs>),
    ExtraSlot(ItemSlot),
}

pub async fn migrate(db_pool: &DbPool, master_store: &MasterStore) -> anyhow::Result<()> {
    let mut tx = db_pool.begin().await?;

    stop_all_grinds(&mut *tx).await?;

    for character_data in read_old_character_data(&mut *tx).await?.into_iter() {
        db::characters_data::save_character_inventory(
            &mut *tx,
            &character_data.character_id,
            &old_player_inventory_to_new(
                &master_store.items_store,
                rmp_serde::from_slice::<OldPlayerInventory>(&character_data.inventory_data)?,
            ),
        )
        .await?;
    }

    tx.commit().await?;
    Ok(())
}

fn old_player_inventory_to_new(
    items_store: &ItemsStore,
    player_inventory: OldPlayerInventory,
) -> player::PlayerInventory {
    player::PlayerInventory {
        equipped: player_inventory
            .equipped
            .into_iter()
            .map(|(item_slot, equipped_slot)| {
                (
                    item_slot,
                    match equipped_slot {
                        OldEquippedSlot::MainSlot(old_item_specs) => {
                            player::EquippedSlot::MainSlot(Box::new(old_item_specs_to_item_specs(
                                items_store,
                                *old_item_specs,
                            )))
                        }
                        OldEquippedSlot::ExtraSlot(item_slot) => {
                            player::EquippedSlot::ExtraSlot(item_slot)
                        }
                    },
                )
            })
            .collect(),
        bag: player_inventory
            .bag
            .into_iter()
            .map(|old_item_specs| old_item_specs_to_item_specs(items_store, old_item_specs))
            .collect(),
        max_bag_size: player_inventory.max_bag_size,
    }
}

fn old_item_specs_to_item_specs(
    items_store: &ItemsStore,
    old_item_specs: OldItemSpecs,
) -> item::ItemSpecs {
    item::ItemSpecs {
        modifiers: ItemModifiers {
            base_item_id: items_store
                .iter()
                .find(|(_, item_base)| item_base.name == old_item_specs.base.name)
                .map(|(item_id, _)| item_id.clone())
                .unwrap_or_default(),
            name: old_item_specs.name,
            rarity: old_item_specs.rarity,
            level: old_item_specs.level,
            affixes: old_item_specs.affixes,
        },
        base: old_item_specs.base,
        weapon_specs: old_item_specs.weapon_specs,
        armor_specs: old_item_specs.armor_specs,
        old_game: old_item_specs.old_game,
    }
}

async fn stop_all_grinds<'c>(executor: impl DbExecutor<'c>) -> anyhow::Result<()> {
    sqlx::query!("DELETE FROM saved_game_instances")
        .execute(executor)
        .await?;
    Ok(())
}

async fn read_old_character_data<'c>(
    executor: impl DbExecutor<'c>,
) -> Result<Vec<CharacterDataEntry>, sqlx::Error> {
    Ok(sqlx::query_as!(
        CharacterDataEntry,
        r#"
        SELECT
            character_id as "character_id: UserCharacterId",
            data_version,
            inventory_data,
            passives_data,
            created_at,
            updated_at
         FROM characters_data
         WHERE data_version = '0.1.0'
         "#
    )
    .fetch_all(executor)
    .await?)
}
