use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};
use sqlx::{Transaction, types::JsonValue};

use shared::data::{
    area::AreaLevel,
    conditional_modifier::Condition,
    item::{ItemModifiers, ItemRarity, ItemSlot},
    item_affix::{AffixEffect, AffixEffectScope, AffixTag, AffixType, ItemAffix},
    skill::{DamageType, RestoreType, SkillType},
    stat_effect::{
        LuckyRollType, MinMax, StatConverterSource, StatConverterSpecs, StatSkillEffectType,
        StatStatusType,
    },
    temple::{Modifier, StatEffect, StatType},
    trigger::HitTrigger,
    user::UserCharacterId,
};

use crate::{
    constants::DATA_VERSION,
    db::{
        characters_data::{CharacterDataEntry, upsert_character_inventory_data},
        pool::{Database, DbExecutor, DbPool},
    },
    game::data::inventory_data::InventoryData,
};

pub async fn migrate(db_pool: &DbPool) -> anyhow::Result<()> {
    let mut tx = db_pool.begin().await?;

    stop_all_grinds(&mut *tx).await?;

    migrate_character_items(&mut tx).await?;
    migrate_stash_items(&mut tx).await?;

    tx.commit().await?;
    Ok(())
}

async fn stop_all_grinds<'c>(executor: impl DbExecutor<'c>) -> anyhow::Result<()> {
    sqlx::query!("DELETE FROM saved_game_instances WHERE data_version <= '0.1.6'")
        .execute(executor)
        .await?;
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
            benedictions_data,
            created_at,
            updated_at
         FROM characters_data
         WHERE data_version = '0.1.6'
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

async fn migrate_stash_items(executor: &mut Transaction<'static, Database>) -> anyhow::Result<()> {
    let market_entries = sqlx::query!(
        r#"
        SELECT
            stash_item_id,
            item_data as "item_data: JsonValue"
        FROM stash_items
        WHERE data_version = '0.1.6'
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
// TODO: Migrate items in inventory, stash_items and stash_item_stats

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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
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
    #[serde(default)]
    pub item_level: AreaLevel,
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
            value: value.value,
            bypass_ignore: value.bypass_ignore,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum OldStatType {
    Life,
    LifeRegen,
    Mana,
    ManaRegen,
    ManaCost {
        #[serde(default)]
        skill_type: Option<SkillType>,
    },
    Armor(Option<DamageType>),
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
    // TODO: Collapse to simple Effect, if more involved trigger is needed, we can always add as pure trigger
    LifeOnHit(#[serde(default)] HitTrigger),
    ManaOnHit(#[serde(default)] HitTrigger),
    Restore(#[serde(default)] Option<RestoreType>),
    CritChance(#[serde(default)] Option<SkillType>),
    CritDamage(#[serde(default)] Option<SkillType>),
    StatusPower(#[serde(default)] Option<StatStatusType>),
    StatusDuration(#[serde(default)] Option<StatStatusType>),
    Speed(#[serde(default)] Option<SkillType>),
    MovementSpeed,
    GoldFind,
    ItemRarity,
    ThreatGain,
    Lucky {
        #[serde(default)]
        skill_type: Option<SkillType>,
        roll_type: LuckyRollType,
    },
    SkillConditionalModifier {
        stat: Box<OldStatType>,
        #[serde(default)]
        skill_type: Option<SkillType>,
        #[serde(default)]
        conditions: Vec<Condition>,
    },
    SkillLevel(#[serde(default)] Option<SkillType>),
    StatConverter(OldStatConverterSpecs),
    StatConditionalModifier {
        stat: Box<OldStatType>,
        #[serde(default)]
        conditions: Vec<Condition>,
    },
    SuccessChance {
        #[serde(default)]
        skill_type: Option<SkillType>,
        #[serde(default)]
        effect_type: Option<OldStatSkillEffectType>,
    },
}

impl From<OldStatType> for StatType {
    fn from(value: OldStatType) -> Self {
        use StatType::*;
        match value {
            OldStatType::Life => Life,
            OldStatType::LifeRegen => LifeRegen,
            OldStatType::Mana => Mana,
            OldStatType::ManaRegen => ManaRegen,
            OldStatType::ManaCost { skill_type } => ManaCost { skill_type },
            OldStatType::Armor(damage_type) => Armor(damage_type),
            OldStatType::DamageResistance {
                skill_type,
                damage_type,
            } => DamageResistance {
                skill_type,
                damage_type,
            },
            OldStatType::TakeFromManaBeforeLife => TakeFromManaBeforeLife,
            OldStatType::Block => Block(Some(SkillType::Attack)),
            OldStatType::BlockSpell => Block(Some(SkillType::Spell)),
            OldStatType::BlockDamageTaken => BlockDamageTaken,
            OldStatType::Damage {
                skill_type,
                damage_type,
            } => Damage {
                skill_type,
                damage_type,
                min_max: None,
            },
            OldStatType::MinDamage {
                skill_type,
                damage_type,
            } => Damage {
                skill_type,
                damage_type,
                min_max: Some(MinMax::Min),
            },
            OldStatType::MaxDamage {
                skill_type,
                damage_type,
            } => Damage {
                skill_type,
                damage_type,
                min_max: Some(MinMax::Max),
            },
            OldStatType::LifeOnHit(hit_trigger) => LifeOnHit {
                skill_type: hit_trigger.skill_type,
            },
            OldStatType::ManaOnHit(hit_trigger) => ManaOnHit {
                skill_type: hit_trigger.skill_type,
            },
            OldStatType::Restore(restore_type) => Restore {
                restore_type,
                skill_type: None,
            },
            OldStatType::CritChance(skill_type) => CritChance(skill_type),
            OldStatType::CritDamage(skill_type) => CritDamage(skill_type),
            OldStatType::StatusPower(stat_status_type) => StatusPower {
                status_type: stat_status_type,
                skill_type: None,
                min_max: None,
            },
            OldStatType::StatusDuration(stat_status_type) => StatusDuration {
                status_type: stat_status_type,
                skill_type: None,
            },
            OldStatType::Speed(skill_type) => Speed(skill_type),
            OldStatType::MovementSpeed => MovementSpeed,
            OldStatType::GoldFind => GoldFind,
            OldStatType::ItemRarity => ItemRarity,
            OldStatType::ThreatGain => ThreatGain,
            OldStatType::Lucky {
                skill_type,
                roll_type,
            } => Lucky {
                skill_type,
                roll_type,
            },
            OldStatType::SkillConditionalModifier {
                stat,
                skill_type,
                conditions,
            } => SkillConditionalModifier {
                stat: Box::new((*stat).into()),
                skill_type,
                conditions,
            },
            OldStatType::SkillLevel(skill_type) => SkillLevel(skill_type),
            OldStatType::StatConverter(stat_converter_specs) => {
                StatConverter(stat_converter_specs.into())
            }
            OldStatType::StatConditionalModifier { stat, conditions } => StatConditionalModifier {
                stat: Box::new((*stat).into()),
                conditions,
            },
            OldStatType::SuccessChance {
                skill_type,
                effect_type,
            } => SuccessChance {
                skill_type,
                effect_type: effect_type.map(Into::into),
            },
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum OldStatSkillEffectType {
    FlatDamage {
        // damage_type: Option<DamageType>,
    },
    ApplyStatus {
        // status_type: Option<StatStatusType>,
    },
    Restore {
        #[serde(default)]
        restore_type: Option<RestoreType>,
    },
    Resurrect,
}

impl From<OldStatSkillEffectType> for StatSkillEffectType {
    fn from(value: OldStatSkillEffectType) -> Self {
        use StatSkillEffectType::*;
        match value {
            OldStatSkillEffectType::FlatDamage {} => FlatDamage {},
            OldStatSkillEffectType::ApplyStatus {} => ApplyStatus { status_type: None },
            OldStatSkillEffectType::Restore { restore_type } => Restore { restore_type },
            OldStatSkillEffectType::Resurrect => Resurrect,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct OldStatConverterSpecs {
    pub source: OldStatConverterSource,
    pub target_stat: Box<OldStatType>,
    pub target_modifier: Modifier,

    #[serde(default)]
    pub is_extra: bool,
    #[serde(default)]
    pub skill_type: Option<SkillType>,
}

impl From<OldStatConverterSpecs> for StatConverterSpecs {
    fn from(value: OldStatConverterSpecs) -> Self {
        Self {
            source: value.source.into(),
            target_stat: Box::new((*value.target_stat).into()),
            target_modifier: value.target_modifier,
            is_extra: value.is_extra,
            skill_type: value.skill_type,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum OldStatConverterSource {
    CritDamage,
    MinDamage {
        #[serde(default)]
        damage_type: Option<DamageType>,
    },
    MaxDamage {
        #[serde(default)]
        damage_type: Option<DamageType>,
    },
    Damage {
        #[serde(default)]
        damage_type: Option<DamageType>,
    },
    ThreatLevel,
    MaxLife,
    MaxMana,
    ManaRegen,
    LifeRegen,
    Block(SkillType),
    // TODO: Add others, like armor, ...
}

impl From<OldStatConverterSource> for StatConverterSource {
    fn from(value: OldStatConverterSource) -> Self {
        use StatConverterSource::*;
        match value {
            OldStatConverterSource::CritDamage => CritDamage,
            OldStatConverterSource::MinDamage { damage_type } => Damage {
                damage_type,
                min_max: Some(MinMax::Min),
            },
            OldStatConverterSource::MaxDamage { damage_type } => Damage {
                damage_type,
                min_max: Some(MinMax::Max),
            },
            OldStatConverterSource::Damage { damage_type } => Damage {
                damage_type,
                min_max: None,
            },
            OldStatConverterSource::ThreatLevel => ThreatLevel,
            OldStatConverterSource::MaxLife => MaxLife,
            OldStatConverterSource::MaxMana => MaxMana,
            OldStatConverterSource::ManaRegen => ManaRegen,
            OldStatConverterSource::LifeRegen => LifeRegen,
            OldStatConverterSource::Block(skill_type) => Block(skill_type),
        }
    }
}
