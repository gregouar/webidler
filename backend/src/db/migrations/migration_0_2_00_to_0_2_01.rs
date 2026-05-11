use std::{collections::HashMap, time::Duration};

use anyhow::Context;
use serde::{Deserialize, Serialize};
use sqlx::Transaction;

use shared::data::{
    area::AreaLevel,
    game_stats::GameStats,
    item::{ItemSlot, ItemSpecs},
    loot::QueuedLoot,
    passive::PassivesTreeState,
    player::{EquippedSlot, PlayerBaseSpecs, PlayerInventory, PlayerResources},
    quest::QuestRewards,
    realms::RealmId,
    user::UserCharacterId,
};

use crate::{
    constants::DATA_VERSION,
    db::{
        characters_data::CharacterDataEntry,
        game_instances::SavedGameInstance,
        pool::{Database, DbPool},
    },
    game::{
        data::inventory_data::{EquippedItemData, InventoryData},
        systems::player_controller::PlayerController,
    },
};

const PREVIOUS_DATA_VERSION: &str = "0.2.00";

pub async fn migrate(db_pool: &DbPool) -> anyhow::Result<()> {
    let mut tx = db_pool.begin().await?;

    migrate_character_data(&mut tx)
        .await
        .context("migrate_character_data")?;
    migrate_saved_game_instances(&mut tx)
        .await
        .context("migrate_saved_game_instances")?;

    tx.commit().await?;
    Ok(())
}

async fn migrate_character_data(
    executor: &mut Transaction<'static, Database>,
) -> anyhow::Result<()> {
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
         WHERE data_version <= $1
         "#,
        PREVIOUS_DATA_VERSION,
    )
    .fetch_all(&mut **executor)
    .await?;

    for character_data in characters_data {
        let inventory_data_bytes = migrate_character_inventory_data(&character_data.inventory_data)
            .context(format!("inventory of '{}'", character_data.character_id))?;

        sqlx::query!(
            "UPDATE characters_data SET
                data_version = $1,
                inventory_data = $2,
                updated_at = CURRENT_TIMESTAMP
             WHERE character_id = $3",
            DATA_VERSION,
            inventory_data_bytes,
            character_data.character_id,
        )
        .execute(&mut **executor)
        .await?;
    }

    Ok(())
}

fn migrate_character_inventory_data(bytes: &[u8]) -> anyhow::Result<Vec<u8>> {
    if let Ok(inventory_data) = rmp_serde::from_slice::<InventoryData>(bytes) {
        return Ok(rmp_serde::to_vec(&inventory_data)?);
    }

    if let Ok(old_inventory) = rmp_serde::from_slice::<OldInventoryData>(bytes) {
        return Ok(rmp_serde::to_vec(&InventoryData::from(old_inventory))?);
    }

    let old_inventory = rmp_serde::from_slice::<OldPlayerInventory>(bytes)?;
    Ok(rmp_serde::to_vec(&old_inventory.into_inventory_data())?)
}

async fn migrate_saved_game_instances(
    executor: &mut Transaction<'static, Database>,
) -> anyhow::Result<()> {
    let game_instances = sqlx::query_as!(
        SavedGameInstance,
        r#"
        SELECT
            character_id as "character_id: UserCharacterId",
            area_id,
            area_level as "area_level: i32",
            saved_at,
            data_version,
            game_data
         FROM saved_game_instances
         WHERE data_version <= $1
         "#,
        PREVIOUS_DATA_VERSION,
    )
    .fetch_all(&mut **executor)
    .await?;

    for instance in game_instances {
        let old_game_data: OldSavedGameData = rmp_serde::from_slice(&instance.game_data)
            .context(format!("saved game of '{}'", instance.character_id))?;
        let game_data = MigratedSavedGameData::from(old_game_data);
        let game_data_bytes = rmp_serde::to_vec(&game_data)?;

        sqlx::query!(
            "UPDATE saved_game_instances SET
                data_version = $1,
                game_data = $2
             WHERE character_id = $3",
            DATA_VERSION,
            game_data_bytes,
            instance.character_id,
        )
        .execute(&mut **executor)
        .await?;
    }

    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OldSavedGameData {
    realm_id: RealmId,
    area_id: String,
    map_item: Option<ItemSpecs>,
    area_level: AreaLevel,
    max_area_level_completed: AreaLevel,
    passives_tree_id: String,
    passives_tree_state: PassivesTreeState,
    player_resources: PlayerResources,
    player_base_specs: PlayerBaseSpecs,
    player_inventory: OldPlayerInventory,
    player_controller: PlayerController,
    queued_loot: Vec<QueuedLoot>,
    game_stats: GameStats,
    last_champion_spawn: AreaLevel,
    auto_progress: bool,
    max_area_level: AreaLevel,
    player_stamina: Duration,

    end_quest: bool,
    quest_rewards: Option<QuestRewards>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MigratedSavedGameData {
    realm_id: RealmId,
    area_id: String,
    map_item: Option<ItemSpecs>,
    area_level: AreaLevel,
    max_area_level_completed: AreaLevel,
    passives_tree_id: String,
    passives_tree_state: PassivesTreeState,
    player_resources: PlayerResources,
    player_base_specs: PlayerBaseSpecs,
    player_inventory: PlayerInventory,
    player_controller: PlayerController,
    queued_loot: Vec<QueuedLoot>,
    game_stats: GameStats,
    last_champion_spawn: AreaLevel,
    auto_progress: bool,
    max_area_level: AreaLevel,
    player_stamina: Duration,

    end_quest: bool,
    quest_rewards: Option<QuestRewards>,
}

impl From<OldSavedGameData> for MigratedSavedGameData {
    fn from(value: OldSavedGameData) -> Self {
        Self {
            realm_id: value.realm_id,
            area_id: value.area_id,
            map_item: value.map_item,
            area_level: value.area_level,
            max_area_level_completed: value.max_area_level_completed,
            passives_tree_id: value.passives_tree_id,
            passives_tree_state: value.passives_tree_state,
            player_resources: value.player_resources,
            player_base_specs: value.player_base_specs,
            player_inventory: value.player_inventory.into(),
            player_controller: value.player_controller,
            queued_loot: value.queued_loot,
            game_stats: value.game_stats,
            last_champion_spawn: value.last_champion_spawn,
            auto_progress: value.auto_progress,
            max_area_level: value.max_area_level,
            player_stamina: value.player_stamina,
            end_quest: value.end_quest,
            quest_rewards: value.quest_rewards,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
struct OldPlayerInventory {
    equipped: HashMap<ItemSlot, OldEquippedSlot>,

    bag: Vec<ItemSpecs>,
    max_bag_size: u8,
}

impl From<OldPlayerInventory> for PlayerInventory {
    fn from(value: OldPlayerInventory) -> Self {
        Self {
            equipped: value
                .equipped
                .into_iter()
                .map(|(item_slot, equipped_slot)| (item_slot, equipped_slot.into()))
                .collect(),
            bag: value.bag,
            max_bag_size: value.max_bag_size,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
struct OldInventoryData {
    equipped: HashMap<ItemSlot, shared::data::item::ItemModifiers>,
    bag: Vec<shared::data::item::ItemModifiers>,
    max_bag_size: u8,
}

impl From<OldInventoryData> for InventoryData {
    fn from(value: OldInventoryData) -> Self {
        Self {
            equipped: value
                .equipped
                .into_iter()
                .map(|(item_slot, modifiers)| {
                    (
                        item_slot,
                        EquippedItemData {
                            modifiers,
                            sheathed: false,
                        },
                    )
                })
                .collect(),
            bag: value.bag,
            max_bag_size: value.max_bag_size,
        }
    }
}

impl OldPlayerInventory {
    fn into_inventory_data(self) -> InventoryData {
        InventoryData {
            equipped: self
                .equipped
                .into_iter()
                .filter_map(|(item_slot, equipped_slot)| match equipped_slot {
                    OldEquippedSlot::MainSlot(item_specs) => {
                        let item_specs = *item_specs;
                        Some((
                            item_slot,
                            EquippedItemData {
                                modifiers: item_specs.modifiers,
                                sheathed: false,
                            },
                        ))
                    }
                    OldEquippedSlot::ExtraSlot(_) => None,
                })
                .collect(),
            bag: self
                .bag
                .into_iter()
                .map(|item_specs| item_specs.modifiers)
                .collect(),
            max_bag_size: self.max_bag_size,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
enum OldEquippedSlot {
    MainSlot(Box<ItemSpecs>),
    ExtraSlot(ItemSlot),
}

impl From<OldEquippedSlot> for EquippedSlot {
    fn from(value: OldEquippedSlot) -> Self {
        match value {
            OldEquippedSlot::MainSlot(item_specs) => EquippedSlot::MainSlot {
                item_specs,
                sheathed: false,
            },
            OldEquippedSlot::ExtraSlot(item_slot) => EquippedSlot::ExtraSlot(item_slot),
        }
    }
}
