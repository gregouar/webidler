use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};
use sqlx::{types::JsonValue, Transaction};

use crate::{
    constants::DATA_VERSION,
    db::{
        characters_data::{upsert_character_inventory_data, CharacterDataEntry, InventoryData},
        pool::{Database, DbExecutor, DbPool},
    },
};

use shared::data::{
    area::AreaLevel,
    character::CharacterSize,
    item::{ItemModifiers, ItemRarity, ItemSlot},
    item_affix::{AffixEffect, AffixEffectScope, AffixTag, AffixType, ItemAffix},
    passive::StatEffect,
    skill::{DamageType, RestoreType, SkillType},
    stat_effect::{Modifier, StatStatusType, StatType},
    trigger::HitTrigger,
    user::UserCharacterId,
};

pub async fn migrate(db_pool: &DbPool) -> anyhow::Result<()> {
    let mut tx = db_pool.begin().await?;

    stop_all_grinds(&mut *tx).await?;

    migrate_market_items(&mut tx).await?;
    migrate_character_items(&mut tx).await?;

    tx.commit().await?;
    Ok(())
}
async fn stop_all_grinds<'c>(executor: impl DbExecutor<'c>) -> anyhow::Result<()> {
    sqlx::query!("DELETE FROM saved_game_instances WHERE data_version = '0.1.2'")
        .execute(executor)
        .await?;
    Ok(())
}

async fn migrate_market_items(executor: &mut Transaction<'static, Database>) -> anyhow::Result<()> {
    sqlx::query!("DELETE FROM market WHERE deleted_at IS NOT NULL")
        .execute(&mut **executor)
        .await?;

    let market_entries = sqlx::query!(
        r#"
        SELECT 
            market_id, 
            item_data as "item_data: JsonValue"
        FROM market
        WHERE data_version IS NULL
        "#
    )
    .fetch_all(&mut **executor)
    .await?;

    let new_market_entries = market_entries
        .into_iter()
        .map(|market_entry| {
            let old_item_modifiers: OldItemModifiers =
                serde_json::from_value(market_entry.item_data)?;
            let item_modifiers: ItemModifiers = old_item_modifiers.into();
            Ok((
                market_entry.market_id,
                serde_json::to_value(&item_modifiers)?,
            ))
        })
        .collect::<anyhow::Result<Vec<_>>>()?;

    for (market_id, item_data) in new_market_entries {
        sqlx::query!(
            "UPDATE market SET item_data = $1, data_version = $2 WHERE market_id = $3",
            item_data,
            DATA_VERSION,
            market_id,
        )
        .execute(&mut **executor)
        .await?;
    }

    Ok(())
}

async fn migrate_character_items(
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
            created_at,
            updated_at
         FROM characters_data
         WHERE data_version = '0.1.2'
         "#,
    )
    .fetch_all(&mut **executor)
    .await?;

    for character_data in characters_data {
        let old_inventory: OldInventoryData =
            rmp_serde::from_slice(&character_data.inventory_data)?;
        let inventory_data: InventoryData = old_inventory.into();
        upsert_character_inventory_data(
            &mut **executor,
            &character_data.character_id,
            rmp_serde::to_vec(&inventory_data)?,
        )
        .await?;
    }

    Ok(())
}

pub type OldDamageMap = HashMap<DamageType, (f64, f64)>;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct OldCharacterSpecs {
    pub name: String,
    pub portrait: String,

    #[serde(default)]
    pub size: CharacterSize,

    #[serde(default)]
    pub position_x: u8,
    #[serde(default)]
    pub position_y: u8,

    pub max_life: f64,
    #[serde(default)]
    pub life_regen: f64,

    #[serde(default)]
    pub max_mana: f64,
    #[serde(default)]
    pub mana_regen: f64,

    #[serde(default)]
    pub take_from_mana_before_life: f32,

    #[serde(default)]
    pub armor: HashMap<DamageType, f64>,
    #[serde(default)]
    pub block: f32,
    #[serde(default)]
    pub block_spell: f32,
    #[serde(default)]
    pub block_damage: f32,

    #[serde(default)]
    pub damage_resistance: HashMap<(SkillType, DamageType), f64>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OldItemModifiers {
    pub base_item_id: String,
    pub name: String,

    pub rarity: ItemRarity,
    pub level: AreaLevel,

    pub affixes: Vec<OldItemAffix>,

    #[serde(default)]
    pub quality: f32,
}

impl From<OldItemModifiers> for ItemModifiers {
    fn from(value: OldItemModifiers) -> Self {
        Self {
            base_item_id: value.base_item_id,
            name: value.name,
            rarity: value.rarity,
            level: value.level,
            affixes: value.affixes.into_iter().map(Into::into).collect(),
            quality: value.quality,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct OldItemAffix {
    pub name: String,
    pub family: String,
    pub tags: HashSet<AffixTag>,

    pub affix_type: AffixType,
    pub tier: u8,

    pub effects: Vec<OldAffixEffect>,
    #[serde(default)] // For retro compatibility
    pub item_level: AreaLevel,
    // pub triggers: Vec<TriggeredEffect>, // TODO
}

impl From<OldItemAffix> for ItemAffix {
    fn from(value: OldItemAffix) -> Self {
        Self {
            name: value.name,
            family: value.family,
            tags: value.tags,
            affix_type: value.affix_type,
            tier: value.tier,
            effects: value.effects.into_iter().map(Into::into).collect(),
            item_level: value.item_level,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct OldAffixEffect {
    pub scope: AffixEffectScope,
    pub stat_effect: OldStatEffect,
}

impl From<OldAffixEffect> for AffixEffect {
    fn from(value: OldAffixEffect) -> Self {
        Self {
            scope: value.scope,
            stat_effect: value.stat_effect.into(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct OldStatEffect {
    pub stat: OldStatType,
    pub modifier: Modifier,
    pub value: f64,

    #[serde(default)]
    pub bypass_ignore: bool,
}

impl From<OldStatEffect> for StatEffect {
    fn from(value: OldStatEffect) -> Self {
        Self {
            stat: value.stat.into(),
            modifier: value.modifier,
            value: match value.modifier {
                Modifier::Multiplier => value.value * 100.0,
                Modifier::Flat => value.value,
            },
            bypass_ignore: value.bypass_ignore,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum OldStatType {
    Life,
    LifeRegen,
    Mana,
    ManaRegen,
    Armor(DamageType),
    DamageResistance {
        #[serde(default)]
        skill_type: Option<SkillType>,
        #[serde(default)]
        damage_type: Option<DamageType>,
    },
    TakeFromManaBeforeLife,
    Block,
    BlockSpell,
    BlockDamageTaken,
    Damage {
        #[serde(default)]
        skill_type: Option<SkillType>,
        #[serde(default)]
        damage_type: Option<DamageType>,
    },
    MinDamage {
        #[serde(default)]
        skill_type: Option<SkillType>,
        #[serde(default)]
        damage_type: Option<DamageType>,
    },
    MaxDamage {
        #[serde(default)]
        skill_type: Option<SkillType>,
        #[serde(default)]
        damage_type: Option<DamageType>,
    },
    LifeOnHit(#[serde(default)] HitTrigger),
    ManaOnHit(#[serde(default)] HitTrigger),
    Restore(#[serde(default)] Option<RestoreType>),
    SpellPower,
    CritChances(#[serde(default)] Option<SkillType>),
    CritDamage(#[serde(default)] Option<SkillType>),
    StatusPower(#[serde(default)] Option<StatStatusType>),
    StatusDuration(#[serde(default)] Option<StatStatusType>),
    Speed(#[serde(default)] Option<SkillType>),
    MovementSpeed,
    GoldFind,
    ThreatGain,
}

impl From<OldStatType> for StatType {
    fn from(value: OldStatType) -> Self {
        match value {
            OldStatType::Life => StatType::Life,
            OldStatType::LifeRegen => StatType::LifeRegen,
            OldStatType::Mana => StatType::Mana,
            OldStatType::ManaRegen => StatType::ManaRegen,
            OldStatType::Armor(damage_type) => StatType::Armor(Some(damage_type)),
            OldStatType::DamageResistance {
                skill_type,
                damage_type,
            } => StatType::DamageResistance {
                skill_type,
                damage_type,
            },
            OldStatType::TakeFromManaBeforeLife => StatType::TakeFromManaBeforeLife,
            OldStatType::Block => StatType::Block,
            OldStatType::BlockSpell => StatType::BlockSpell,
            OldStatType::BlockDamageTaken => StatType::BlockDamageTaken,
            OldStatType::Damage {
                skill_type,
                damage_type,
            } => StatType::Damage {
                skill_type,
                damage_type,
            },
            OldStatType::MinDamage {
                skill_type,
                damage_type,
            } => StatType::MinDamage {
                skill_type,
                damage_type,
            },
            OldStatType::MaxDamage {
                skill_type,
                damage_type,
            } => StatType::MaxDamage {
                skill_type,
                damage_type,
            },
            OldStatType::LifeOnHit(hit_trigger) => StatType::LifeOnHit(hit_trigger),
            OldStatType::ManaOnHit(hit_trigger) => StatType::ManaOnHit(hit_trigger),
            OldStatType::Restore(restore_type) => StatType::Restore(restore_type),
            OldStatType::SpellPower => StatType::Damage {
                skill_type: Some(SkillType::Spell),
                damage_type: None,
            },
            OldStatType::CritChances(skill_type) => StatType::CritChance(skill_type),
            OldStatType::CritDamage(skill_type) => StatType::CritDamage(skill_type),
            OldStatType::StatusPower(stat_status_type) => StatType::StatusPower(stat_status_type),
            OldStatType::StatusDuration(stat_status_type) => {
                StatType::StatusDuration(stat_status_type)
            }
            OldStatType::Speed(skill_type) => StatType::Speed(skill_type),
            OldStatType::MovementSpeed => StatType::MovementSpeed,
            OldStatType::GoldFind => StatType::GoldFind,
            OldStatType::ThreatGain => StatType::ThreatGain,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct OldInventoryData {
    pub equipped: HashMap<ItemSlot, OldItemModifiers>,
    pub bag: Vec<OldItemModifiers>,
    pub max_bag_size: u8,
}

impl From<OldInventoryData> for InventoryData {
    fn from(value: OldInventoryData) -> Self {
        Self {
            equipped: value
                .equipped
                .into_iter()
                .map(|(item_slot, item_modifiers)| (item_slot, item_modifiers.into()))
                .collect(),
            bag: value.bag.into_iter().map(Into::into).collect(),
            max_bag_size: value.max_bag_size,
        }
    }
}
