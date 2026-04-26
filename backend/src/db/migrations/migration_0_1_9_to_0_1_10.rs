use std::collections::{BTreeSet, HashMap};

use anyhow::Context;
use serde::{Deserialize, Serialize};
use sqlx::{Transaction, types::JsonValue};

use shared::data::{
    area::AreaLevel,
    conditional_modifier::Condition,
    item::{ItemModifiers, ItemRarity, ItemSlot, SkillRange, SkillShape},
    item_affix::{AffixEffect, AffixEffectScope, AffixTag, AffixType, ItemAffix},
    modifier::Modifier,
    passive::PassivesTreeAscension,
    skill::{DamageType, RestoreType, SkillType},
    stat_effect::{
        ArmorStatType, LuckyRollType, MinMax, StatConverterSource, StatConverterSpecs, StatEffect,
        StatSkillEffectType, StatSkillFilter, StatSkillRepeat, StatStatusType, StatType,
    },
    temple::PlayerBenedictions,
    user::UserCharacterId,
};

use crate::{
    app_state::MasterStore,
    constants::DATA_VERSION,
    db::{
        self,
        characters_data::{CharacterDataEntry, upsert_character_inventory_data},
        pool::{Database, DbExecutor, DbPool},
    },
    game::{
        data::inventory_data::InventoryData,
        systems::{benedictions_controller, passives_controller},
    },
};

pub async fn migrate(db_pool: &DbPool, master_store: &MasterStore) -> anyhow::Result<()> {
    let mut tx = db_pool.begin().await?;

    stop_all_grinds(&mut *tx).await?;
    clear_game_stats(&mut *tx).await?;

    migrate_character_data(&mut tx, master_store)
        .await
        .context("migrate_character_data")?;
    migrate_stash_items(&mut tx)
        .await
        .context("migrate_stash_items")?;

    tx.commit().await?;
    Ok(())
}

async fn stop_all_grinds<'c>(executor: impl DbExecutor<'c>) -> anyhow::Result<()> {
    sqlx::query!("DELETE FROM saved_game_instances WHERE data_version <= '0.1.9'")
        .execute(executor)
        .await?;
    Ok(())
}

async fn clear_game_stats<'c>(executor: impl DbExecutor<'c>) -> anyhow::Result<()> {
    sqlx::query!("DELETE FROM game_stats WHERE data_version <= '0.1.9'")
        .execute(executor)
        .await?;
    Ok(())
}

async fn migrate_character_data(
    executor: &mut Transaction<'static, Database>,
    master_store: &MasterStore,
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
         WHERE data_version <= '0.1.9'
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

        let character =
            db::characters::read_character(&mut **executor, &character_data.character_id)
                .await?
                .ok_or(anyhow::anyhow!("character not found"))?;

        passives_controller::update_ascension(
            executor,
            master_store,
            &character_data.character_id,
            character.resource_shards,
            &PassivesTreeAscension::default(),
        )
        .await?;

        benedictions_controller::update_benedictions(
            executor,
            master_store,
            &character_data.character_id,
            character.resource_gold,
            &PlayerBenedictions::default(),
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
        WHERE data_version <= '0.1.9'
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
    SkillLevel(#[serde(default)] Option<SkillType>),
    Armor(Option<DamageType>),
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
        #[serde(default)]
        skill_type: Option<SkillType>,
        #[serde(default)]
        damage_type: Option<DamageType>,
        #[serde(default)]
        min_max: Option<MinMax>,
    },
    CritChance(#[serde(default)] Option<SkillType>),
    CritDamage(#[serde(default)] Option<SkillType>),
    StatusPower {
        #[serde(default)]
        status_type: Option<OldStatStatusType>,
        #[serde(default)]
        skill_type: Option<SkillType>,
        #[serde(default)]
        min_max: Option<MinMax>,
    },
    StatusDuration {
        #[serde(default)]
        status_type: Option<OldStatStatusType>,
        #[serde(default)]
        skill_type: Option<SkillType>,
    },
    Speed(#[serde(default)] Option<SkillType>),
    RestoreOnHit {
        restore_type: RestoreType,
        #[serde(default)]
        skill_type: Option<SkillType>,
    },
    Restore {
        #[serde(default)]
        restore_type: Option<RestoreType>,
        #[serde(default)]
        skill_type: Option<SkillType>,
    },
    Life,
    LifeRegen,
    Mana,
    ManaRegen,
    ManaCost {
        #[serde(default)]
        skill_type: Option<SkillType>,
    },
    TakeFromManaBeforeLife,
    TakeFromLifeBeforeMana,
    MovementSpeed,
    ThreatGain,
    Lucky {
        #[serde(default)]
        skill_type: Option<SkillType>,
        roll_type: OldLuckyRollType,
    },
    SuccessChance {
        #[serde(default)]
        skill_type: Option<SkillType>,
        #[serde(default)]
        effect_type: Option<OldStatSkillEffectType>,
    },
    SkillConditionalModifier {
        stat: Box<OldStatType>,
        #[serde(default)]
        skill_type: Option<SkillType>,
        #[serde(default)]
        conditions: Vec<OldCondition>,
    },
    SkillTargetModifier {
        // TODO: More control and options?
        #[serde(default)]
        skill_type: Option<SkillType>,
        #[serde(default)]
        range: Option<SkillRange>,
        #[serde(default)]
        shape: Option<SkillShape>,
        #[serde(default)]
        repeat: Option<StatSkillRepeat>,
        #[serde(default)]
        skill_id: Option<String>,
    },
    StatConditionalModifier {
        stat: Box<OldStatType>,
        conditions: Vec<OldCondition>,
        #[serde(default)]
        conditions_duration: u32,
    },
    StatConverter(OldStatConverterSpecs),
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
            OldStatType::ManaCost { skill_type } => ManaCost {
                skill_filter: StatSkillFilter {
                    skill_type,
                    skill_id: None,
                    skill_description: None,
                },
            },
            OldStatType::Armor(damage_type) => {
                Armor(damage_type.map(|damage_type| match damage_type {
                    DamageType::Physical => ArmorStatType::Physical,
                    DamageType::Fire => ArmorStatType::Fire,
                    DamageType::Poison => ArmorStatType::Poison,
                    DamageType::Storm => ArmorStatType::Storm,
                }))
            }
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
                skill_type,
                damage_type,
                min_max,
            } => Damage {
                skill_filter: StatSkillFilter {
                    skill_type,
                    skill_id: None,
                    skill_description: None,
                },
                damage_type,
                min_max,
                is_hit: None,
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
                skill_type,
            } => Restore {
                restore_type,
                skill_filter: StatSkillFilter {
                    skill_type,
                    skill_id: None,
                    skill_description: None,
                },
            },
            OldStatType::CritChance(skill_type) => CritChance(StatSkillFilter {
                skill_type,
                skill_id: None,
                skill_description: None,
            }),
            OldStatType::CritDamage(skill_type) => CritDamage(StatSkillFilter {
                skill_type,
                skill_id: None,
                skill_description: None,
            }),
            OldStatType::StatusPower {
                status_type,
                skill_type,
                min_max,
            } => StatusPower {
                status_type: status_type.map(|status_type| status_type.into()),
                skill_filter: StatSkillFilter {
                    skill_type,
                    skill_id: None,
                    skill_description: None,
                },
                min_max,
            },
            OldStatType::StatusDuration {
                status_type,
                skill_type,
            } => StatusDuration {
                status_type: status_type.map(|status_type| status_type.into()),
                skill_filter: StatSkillFilter {
                    skill_type,
                    skill_id: None,
                    skill_description: None,
                },
            },
            OldStatType::Speed(skill_type) => Speed(StatSkillFilter {
                skill_type,
                skill_id: None,
                skill_description: None,
            }),
            OldStatType::MovementSpeed => MovementSpeed,
            OldStatType::GoldFind => GoldFind,
            OldStatType::ItemRarity => ItemRarity,
            OldStatType::ThreatGain => ThreatGain,
            OldStatType::Lucky {
                skill_type,
                roll_type,
            } => Lucky {
                skill_filter: StatSkillFilter {
                    skill_type,
                    skill_id: None,
                    skill_description: None,
                },
                roll_type: roll_type.into(),
            },
            OldStatType::SkillConditionalModifier {
                stat,
                skill_type,
                conditions,
            } => SkillConditionalModifier {
                stat: Box::new((*stat).into()),
                skill_filter: StatSkillFilter {
                    skill_type,
                    skill_id: None,
                    skill_description: None,
                },
                conditions: conditions.into_iter().map(|c| c.into()).collect(),
            },
            OldStatType::SkillLevel(skill_type) => SkillLevel(StatSkillFilter {
                skill_type,
                skill_id: None,
                skill_description: None,
            }),
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
                skill_type,
                effect_type,
            } => SuccessChance {
                skill_filter: StatSkillFilter {
                    skill_type,
                    skill_id: None,
                    skill_description: None,
                },
                effect_type: effect_type.map(Into::into),
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
                status_type: status_type.map(|status_type| status_type.into()),
            },
            OldStatType::TakeFromLifeBeforeMana => TakeFromLifeBeforeMana,
            OldStatType::SkillTargetModifier {
                skill_type,
                range,
                shape,
                repeat,
                skill_id,
            } => SkillTargetModifier {
                skill_filter: StatSkillFilter {
                    skill_type,
                    skill_id,
                    skill_description: None,
                },
                range,
                shape,
                repeat,
            },
            OldStatType::Description2(d) => Description2(d),
            OldStatType::PowerLevel => PowerLevel,
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
        #[serde(default)]
        trigger_description: Option<String>,
    },
}

impl From<OldStatStatusType> for StatStatusType {
    fn from(value: OldStatStatusType) -> Self {
        use StatStatusType::*;
        match value {
            OldStatStatusType::Stun => Stun,
            OldStatStatusType::DamageOverTime { damage_type } => DamageOverTime { damage_type },
            OldStatStatusType::StatModifier { debuff, stat } => StatModifier {
                debuff,
                stat: stat.map(|stat| Box::new((*stat).into())),
            },
            OldStatStatusType::Trigger {
                trigger_id,
                trigger_description,
            } => Trigger {
                trigger_id,
                trigger_description,
            },
        }
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
}

impl From<OldStatSkillEffectType> for StatSkillEffectType {
    fn from(value: OldStatSkillEffectType) -> Self {
        use StatSkillEffectType::*;
        match value {
            OldStatSkillEffectType::FlatDamage {} => FlatDamage {},
            OldStatSkillEffectType::ApplyStatus { status_type } => ApplyStatus {
                status_type: status_type.map(|status_type| status_type.into()),
            },
            OldStatSkillEffectType::Restore { restore_type } => Restore { restore_type },
            OldStatSkillEffectType::Resurrect => Resurrect,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct OldStatConverterSpecs {
    pub source: StatConverterSource,
    pub target_stat: Box<OldStatType>,

    #[serde(default)]
    pub is_extra: bool,
    #[serde(default)]
    pub skill_type: Option<SkillType>,
}

impl From<OldStatConverterSpecs> for StatConverterSpecs {
    fn from(value: OldStatConverterSpecs) -> Self {
        Self {
            source: value.source,
            stat: Box::new((*value.target_stat).into()),
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
                status_type: status_type.map(|status_type| status_type.into()),
                skill_type,
                not,
            },
            OldCondition::StatusStacks {
                status_type,
                skill_type,
            } => StatusStacks {
                status_type: status_type.map(|status_type| status_type.into()),
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
