use std::collections::{BTreeSet, HashMap, HashSet};

use anyhow::Context;
use serde::{Deserialize, Serialize};
use sqlx::{Transaction, types::JsonValue};

use shared::data::{
    area::AreaLevel,
    character_status::StatusId,
    conditional_modifier::Condition,
    item::{ItemModifiers, ItemRarity, ItemSlot, SkillRange, SkillShape},
    item_affix::{AffixEffect, AffixEffectScope, AffixTag, AffixType, ItemAffix},
    modifier::Modifier,
    skill::{DamageType, RestoreType, SkillType},
    stat_effect::{
        ArmorStatType, LuckyRollType, MinMax, StatConverterSource, StatConverterSpecs, StatEffect,
        StatSkillEffectType, StatSkillFilter, StatSkillRepeat, StatStatusFilter, StatType,
    },
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

    migrate_character_data(&mut tx)
        .await
        .context("migrate_character_data")?;
    migrate_stash_items(&mut tx)
        .await
        .context("migrate_stash_items")?;

    tx.commit().await?;
    Ok(())
}

async fn stop_all_grinds<'c>(executor: impl DbExecutor<'c>) -> anyhow::Result<()> {
    sqlx::query!("DELETE FROM saved_game_instances WHERE data_version <= '0.2.00'")
        .execute(executor)
        .await?;
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
         WHERE data_version <= '0.2.00'
         "#,
    )
    .fetch_all(&mut **executor)
    .await?;

    for character_data in characters_data {
        let old_inventory: OldInventoryData = rmp_serde::from_slice(&character_data.inventory_data)
            .context(format!("inventory of '{}'", character_data.character_id))?;
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
        WHERE data_version <= '0.2.00'
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

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct OldInventoryData {
    pub equipped: HashMap<ItemSlot, OldItemModifiers>,
    pub bag: Vec<OldItemModifiers>,
    pub max_bag_size: u8,

    #[serde(default)]
    pub sheathed: HashSet<ItemSlot>,
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
            sheathed: value.sheathed,
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

    #[serde(default)]
    pub upgrade_level: u8,
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
            upgrade_level: value.upgrade_level,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct OldItemAffix {
    pub name: String,
    pub family: String,
    pub tags: BTreeSet<AffixTag>,

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
    Description(String),
    GemsFind,
    ItemRarity,
    ItemLevel,
    SkillLevel(#[serde(default)] OldStatSkillFilter),
    Armor(Option<ArmorStatType>),
    DamageResistance {
        #[serde(default)]
        skill_type: Option<SkillType>,
        #[serde(default)]
        damage_type: Option<DamageType>,
    },
    Block(#[serde(default)] Option<SkillType>),
    BlockDamageTaken,
    Evade(#[serde(default)] Option<DamageType>),
    EvadeDamageTaken,
    StatusResistance {
        #[serde(default)]
        skill_type: Option<SkillType>,
        #[serde(default)]
        status_type: Option<OldStatStatusType>,
    },
    Damage {
        #[serde(flatten)]
        skill_filter: OldStatSkillFilter,
        #[serde(default)]
        damage_type: Option<DamageType>,
        #[serde(default)]
        min_max: Option<MinMax>,
        #[serde(default)]
        is_hit: Option<bool>,
    },
    CritChance(#[serde(default)] OldStatSkillFilter),
    CritDamage(#[serde(default)] OldStatSkillFilter),
    StatusPower {
        #[serde(default)]
        status_type: Option<OldStatStatusType>,
        #[serde(flatten)]
        skill_filter: OldStatSkillFilter,
        #[serde(default)]
        min_max: Option<MinMax>,
    },
    StatusDuration {
        #[serde(default)]
        status_type: Option<OldStatStatusType>,
        #[serde(flatten)]
        skill_filter: OldStatSkillFilter,
    },
    StatusEscalation {
        #[serde(flatten)]
        skill_filter: OldStatSkillFilter,
        #[serde(default)]
        damage_type: Option<DamageType>,
    },
    StatusFaster {
        #[serde(flatten)]
        skill_filter: OldStatSkillFilter,
        #[serde(default)]
        damage_type: Option<DamageType>,
    },
    SuccessChance {
        #[serde(flatten)]
        skill_filter: OldStatSkillFilter,
        #[serde(default)]
        effect_type: Option<OldStatSkillEffectType>,
    },
    Speed(#[serde(default)] OldStatSkillFilter),
    RestoreOnHit {
        restore_type: RestoreType,
        #[serde(default)]
        skill_type: Option<SkillType>,
    },
    Restore {
        #[serde(default)]
        restore_type: Option<RestoreType>,
        #[serde(flatten)]
        skill_filter: OldStatSkillFilter,
    },
    Life,
    LifeRegen,
    Mana,
    ManaRegen,
    ManaCost {
        #[serde(flatten)]
        skill_filter: OldStatSkillFilter,
    },
    TakeFromManaBeforeLife,
    TakeFromLifeBeforeMana,
    MovementSpeed,
    ThreatGain,
    Lucky {
        #[serde(flatten)]
        skill_filter: OldStatSkillFilter,
        roll_type: OldLuckyRollType,
    },
    SkillConditionalModifier {
        stat: Box<OldStatType>,
        #[serde(flatten)]
        skill_filter: OldStatSkillFilter,
        #[serde(default)]
        conditions: Vec<OldCondition>,
    },
    StatConditionalModifier {
        stat: Box<OldStatType>,
        conditions: Vec<OldCondition>,
        #[serde(default)]
        conditions_duration: u32,
    },
    StatConverter(OldStatConverterSpecs),
    SkillTargetModifier {
        // TODO: More control and options?
        #[serde(flatten)]
        skill_filter: OldStatSkillFilter,
        #[serde(default)]
        range: Option<SkillRange>,
        #[serde(default)]
        shape: Option<SkillShape>,
        #[serde(default)]
        repeat: Option<StatSkillRepeat>,
    },
    GoldFind,
    PowerLevel,
    Description2(String),
}

impl From<OldStatType> for StatType {
    fn from(value: OldStatType) -> Self {
        use StatType::*;
        match value {
            OldStatType::Life => Life,
            OldStatType::LifeRegen => LifeRegen,
            OldStatType::Mana => Mana,
            OldStatType::ManaRegen => ManaRegen,
            OldStatType::ManaCost { skill_filter } => ManaCost {
                skill_filter: skill_filter.into(),
            },
            OldStatType::Armor(damage_type) => Armor(damage_type),
            OldStatType::DamageResistance {
                skill_type,
                damage_type,
            } => DamageResistance {
                skill_type,
                damage_type,
            },
            OldStatType::TakeFromManaBeforeLife => TakeFromManaBeforeLife,
            OldStatType::Block(skill_type) => Block(skill_type),
            OldStatType::BlockDamageTaken => BlockDamageTaken,
            OldStatType::Damage {
                skill_filter,
                damage_type,
                min_max,
                is_hit,
            } => Damage {
                skill_filter: skill_filter.into(),
                damage_type,
                min_max,
                is_hit,
            },
            OldStatType::RestoreOnHit {
                restore_type,
                skill_type,
            } => RestoreOnHit {
                restore_type,
                skill_type,
            },
            OldStatType::Restore {
                restore_type,
                skill_filter,
            } => Restore {
                restore_type,
                skill_filter: skill_filter.into(),
            },
            OldStatType::CritChance(skill_filter) => CritChance(skill_filter.into()),
            OldStatType::CritDamage(skill_filter) => CritDamage(skill_filter.into()),
            OldStatType::StatusPower {
                status_type,
                skill_filter,
                min_max,
            } => StatusPower {
                status_filter: status_type_to_status_filter(status_type),
                skill_filter: skill_filter.into(),
                min_max,
            },
            OldStatType::StatusDuration {
                status_type,
                skill_filter,
            } => StatusDuration {
                status_filter: status_type_to_status_filter(status_type),
                skill_filter: skill_filter.into(),
            },
            OldStatType::Speed(skill_filter) => Speed(skill_filter.into()),
            OldStatType::MovementSpeed => MovementSpeed,
            OldStatType::GoldFind => GoldFind,
            OldStatType::ItemRarity => ItemRarity,
            OldStatType::ThreatGain => ThreatGain,
            OldStatType::Lucky {
                skill_filter,
                roll_type,
            } => Lucky {
                skill_filter: skill_filter.into(),
                roll_type: roll_type.into(),
            },
            OldStatType::SkillConditionalModifier {
                stat,
                skill_filter,
                conditions,
            } => SkillConditionalModifier {
                stat: Box::new((*stat).into()),
                skill_filter: skill_filter.into(),
                conditions: conditions.into_iter().map(|c| c.into()).collect(),
            },
            OldStatType::SkillLevel(skill_filter) => SkillLevel(skill_filter.into()),
            OldStatType::StatConverter(stat_converter_specs) => {
                StatConverter(stat_converter_specs.into())
            }
            OldStatType::StatConditionalModifier {
                stat,
                conditions,
                conditions_duration,
            } => StatConditionalModifier {
                stat: Box::new((*stat).into()),
                conditions: conditions.into_iter().map(|c| c.into()).collect(),
                conditions_duration,
            },
            OldStatType::SuccessChance {
                skill_filter,
                effect_type,
            } => SuccessChance {
                skill_filter: skill_filter.into(),
                effect_type: effect_type.map(|e| e.into()),
            },
            OldStatType::Description(d) => Description(d),
            OldStatType::GemsFind => GemsFind,
            OldStatType::ItemLevel => ItemLevel,
            OldStatType::Evade(damage_type) => Evade(damage_type),
            OldStatType::EvadeDamageTaken => EvadeDamageTaken,
            OldStatType::StatusResistance {
                skill_type,
                status_type,
            } => StatusResistance {
                skill_type,
                status_id: status_type.map(status_type_to_status_id),
            },
            OldStatType::TakeFromLifeBeforeMana => TakeFromLifeBeforeMana,
            OldStatType::SkillTargetModifier {
                skill_filter,
                range,
                shape,
                repeat,
            } => SkillTargetModifier {
                skill_filter: skill_filter.into(),
                range,
                shape,
                repeat,
            },
            OldStatType::Description2(d) => Description2(d),
            OldStatType::PowerLevel => PowerLevel,
            OldStatType::StatusEscalation {
                skill_filter,
                damage_type,
            } => StatusEscalation {
                status_filter: StatStatusFilter {
                    status_id: None,
                    damage_type: damage_type.map(|d| d.into()),
                },
                skill_filter: skill_filter.into(),
            },
            OldStatType::StatusFaster {
                skill_filter,
                damage_type,
            } => StatusFaster {
                status_filter: StatStatusFilter {
                    status_id: None,
                    damage_type: damage_type.map(|d| d.into()),
                },
                skill_filter: skill_filter.into(),
            },
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum OldStatStatusType {
    Stun,
    DamageOverTime {
        #[serde(default)]
        damage_type: Option<DamageType>,
    },
    StatModifier {
        #[serde(default)]
        debuff: Option<bool>,
        #[serde(default)]
        stat: Option<Box<OldStatType>>,
    },
    Trigger {
        #[serde(default)]
        trigger_id: Option<String>,

        // TODO: This is awful....
        #[serde(default)]
        trigger_description: Option<String>,
    },
}

fn status_type_to_status_filter(value: Option<OldStatStatusType>) -> StatStatusFilter {
    match value {
        Some(value) => match value {
            OldStatStatusType::Stun => StatStatusFilter {
                status_id: Some("stun".into()),
                damage_type: None,
            },
            OldStatStatusType::DamageOverTime { damage_type } => StatStatusFilter {
                status_id: None,
                damage_type: damage_type.map(|d| d.into()),
            },
            OldStatStatusType::StatModifier { .. } => StatStatusFilter {
                status_id: None,
                damage_type: None,
            },
            OldStatStatusType::Trigger { trigger_id, .. } => StatStatusFilter {
                status_id: trigger_id.map(|x| x.into()),
                damage_type: None,
            },
        },
        None => Default::default(),
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum OldStatSkillEffectType {
    FlatDamage {
        // damage_type: Option<DamageType>,
    },
    ApplyStatus {
        status_type: Option<OldStatStatusType>,
    },
    Restore {
        #[serde(default)]
        restore_type: Option<RestoreType>,
    },
    Resurrect,
    RefreshCooldown,
}

impl From<OldStatSkillEffectType> for StatSkillEffectType {
    fn from(value: OldStatSkillEffectType) -> Self {
        use StatSkillEffectType::*;
        match value {
            OldStatSkillEffectType::FlatDamage {} => FlatDamage {},
            OldStatSkillEffectType::ApplyStatus { status_type } => ApplyStatus {
                status_id: status_type.map(status_type_to_status_id),
            },
            OldStatSkillEffectType::Restore { restore_type } => Restore { restore_type },
            OldStatSkillEffectType::Resurrect => Resurrect,
            OldStatSkillEffectType::RefreshCooldown => RefreshCooldown,
        }
    }
}

fn status_type_to_status_id(status_type: OldStatStatusType) -> StatusId {
    match status_type {
        OldStatStatusType::Stun => "stun".into(),
        OldStatStatusType::DamageOverTime { damage_type } => match damage_type {
            Some(damage_type) => match damage_type {
                DamageType::Physical => "bleed",
                DamageType::Fire => "burn",
                DamageType::Poison => "poison",
                DamageType::Storm => "",
            },
            None => "",
        }
        .into(),
        OldStatStatusType::StatModifier { .. } => "".into(),
        OldStatStatusType::Trigger { trigger_id, .. } => trigger_id.unwrap_or_default(),
    }
    .into()
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct OldStatConverterSpecs {
    pub source: StatConverterSource,
    pub stat: Box<OldStatType>,

    #[serde(default)]
    pub is_extra: bool,
    #[serde(default)]
    pub skill_type: Option<SkillType>,
}

impl From<OldStatConverterSpecs> for StatConverterSpecs {
    fn from(value: OldStatConverterSpecs) -> Self {
        Self {
            source: value.source,
            stat: Box::new((*value.stat).into()),
            is_extra: value.is_extra,
            skill_type: value.skill_type,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum OldLuckyRollType {
    Damage {
        #[serde(default)]
        damage_type: Option<DamageType>,
    },
    Block,
    Evade(Option<DamageType>),
    CritChance,
    SuccessChance {
        #[serde(default)]
        effect_type: Option<OldStatSkillEffectType>,
    },
}

impl From<OldLuckyRollType> for LuckyRollType {
    fn from(value: OldLuckyRollType) -> Self {
        use LuckyRollType::*;
        match value {
            OldLuckyRollType::Damage { damage_type } => Damage { damage_type },
            OldLuckyRollType::Block => Block,
            OldLuckyRollType::Evade(damage_type) => Evade(damage_type),
            OldLuckyRollType::CritChance => CritChance,
            OldLuckyRollType::SuccessChance { effect_type } => SuccessChance {
                effect_type: effect_type.map(|effect_type| effect_type.into()),
            },
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum OldCondition {
    // HitCrit,
    HasStatus {
        #[serde(default)]
        status_type: Option<OldStatStatusType>,
        #[serde(default)]
        skill_type: Option<SkillType>,
        #[serde(default)]
        not: bool,
    },
    StatusStacks {
        #[serde(default)]
        status_type: Option<OldStatStatusType>,
        #[serde(default)]
        skill_type: Option<SkillType>,
    },
    // StatusValue(Option<StatStatusType>),
    // StatusDuration(Option<StatStatusType>),
    MaximumLife,
    MaximumMana,
    LowLife,
    LowMana,
    ThreatLevel,
}

impl From<OldCondition> for Condition {
    fn from(value: OldCondition) -> Self {
        use Condition::*;
        match value {
            OldCondition::HasStatus {
                status_type,
                skill_type,
                not,
            } => HasStatus {
                status_filter: status_type_to_status_filter(status_type),
                skill_type,
                not,
            },
            OldCondition::StatusStacks {
                status_type,
                skill_type,
            } => StatusStacks {
                status_filter: status_type_to_status_filter(status_type),
                skill_type,
            },
            OldCondition::MaximumLife => MaximumLife,
            OldCondition::MaximumMana => MaximumMana,
            OldCondition::LowLife => LowLife,
            OldCondition::LowMana => LowMana,
            OldCondition::ThreatLevel => ThreatLevel,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct OldStatSkillFilter {
    #[serde(default)]
    pub skill_type: Option<SkillType>,

    #[serde(default)]
    pub skill_id: Option<String>,
    // TODO: This is awful....
    #[serde(default)]
    pub skill_description: Option<String>,
}

impl From<OldStatSkillFilter> for StatSkillFilter {
    fn from(value: OldStatSkillFilter) -> Self {
        StatSkillFilter {
            skill_type: value.skill_type,
            skill_id: value.skill_id,
        }
    }
}
