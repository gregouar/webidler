use std::collections::{BTreeSet, HashMap, HashSet};

use anyhow::Context;
use serde::{Deserialize, Serialize};
use sqlx::{Transaction, types::JsonValue};

use shared::data::{
    area::AreaLevel,
    character_status::StatusId,
    conditional_modifier::Condition,
    item::{
        AffixRoll, ItemBase, ItemCategory, ItemModifiers, ItemRarity, ItemSlot, SkillRange,
        SkillShape,
    },
    item_affix::{AffixEffect, AffixEffectBlueprint, AffixEffectScope, AffixTag, AffixType},
    modifier::Modifier,
    passive::PassiveNodeId,
    skill::{DamageType, RestoreModifier, RestoreType, SkillType},
    stat_effect::{
        DamageStatType, LuckyRollType, MinMax, StatConverterSource, StatConverterSpecs, StatEffect,
        StatSkillEffectType, StatSkillFilter, StatSkillRepeat, StatStatusFilter, StatType,
    },
    trigger::TriggerEffectModifierSource,
    user::UserCharacterId,
};

use crate::{
    app_state::MasterStore,
    db::{
        characters_data::CharacterDataEntry,
        pool::{Database, DbExecutor, DbPool},
    },
    game::{
        data::{
            inventory_data::InventoryData, items_store::ItemsStore,
            passives::PassivesTreeAscensionData,
        },
        utils::rng,
    },
};

use super::realm_rollover;

const TARGET_DATA_VERSION: &str = "0.4.00";
const REALM_MIGRATIONS: [(&str, &str); 2] = [("Standard", "Legacy"), ("StandardSSF", "LegacySSF")];

pub async fn migrate(db_pool: &DbPool, master_store: &MasterStore) -> anyhow::Result<()> {
    let mut tx = db_pool.begin().await?;

    stop_all_grinds(&mut *tx).await?;
    if !migration_required(&mut *tx).await? {
        tx.commit().await?;
        return Ok(());
    }

    realm_rollover::rollover_realms(&mut tx, &REALM_MIGRATIONS, TARGET_DATA_VERSION)
        .await
        .context("rollover_realms")?;
    migrate_character_items(&mut tx, &master_store.items_store)
        .await
        .context("migrate_character_items")?;
    migrate_stash_and_market_items(&mut tx, &master_store.items_store)
        .await
        .context("migrate_stash_and_market_items")?;
    realm_rollover::finalize_data_versions(&mut tx, TARGET_DATA_VERSION)
        .await
        .context("finalize_data_versions")?;

    tx.commit().await?;
    Ok(())
}

async fn migration_required<'c>(executor: impl DbExecutor<'c>) -> anyhow::Result<bool> {
    let pending_rows = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) as "count!"
        FROM (
            SELECT data_version FROM characters_data WHERE data_version < $1
            UNION ALL
            SELECT data_version FROM stash_items WHERE data_version < $1
            UNION ALL
            SELECT data_version FROM stashes WHERE data_version < $1
            UNION ALL
            SELECT data_version FROM game_stats WHERE data_version < $1
            UNION ALL
            SELECT data_version FROM leaderboard WHERE data_version < $1
        ) AS pending_migrations
        "#,
        TARGET_DATA_VERSION,
    )
    .fetch_one(executor)
    .await?;

    Ok(pending_rows > 0)
}

async fn stop_all_grinds<'c>(executor: impl DbExecutor<'c>) -> anyhow::Result<()> {
    sqlx::query!(
        "DELETE FROM saved_game_instances WHERE data_version < $1",
        TARGET_DATA_VERSION
    )
    .execute(executor)
    .await?;
    Ok(())
}

async fn migrate_character_items(
    executor: &mut Transaction<'static, Database>,
    items_store: &ItemsStore,
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
            skill_masteries_data,
            pets_data,
            created_at,
            updated_at
         FROM characters_data
         WHERE data_version < $1
         "#,
        TARGET_DATA_VERSION
    )
    .fetch_all(&mut **executor)
    .await?;

    for character_data in characters_data {
        let old_inventory: OldInventoryData = rmp_serde::from_slice(&character_data.inventory_data)
            .with_context(|| format!("inventory of '{}'", character_data.character_id))?;
        let inventory = InventoryData {
            equipped: old_inventory
                .equipped
                .into_iter()
                .map(|(slot, item)| migrate_item(item, items_store).map(|item| (slot, item)))
                .collect::<anyhow::Result<_>>()?,
            bag: old_inventory
                .bag
                .into_iter()
                .map(|item| migrate_item(item, items_store))
                .collect::<anyhow::Result<_>>()?,
            max_bag_size: old_inventory.max_bag_size,
            sheathed: old_inventory.sheathed,
        };

        let passives_data = if let Some(passives_data) = character_data.passives_data {
            let old_passives: OldPassivesTreeAscensionData = rmp_serde::from_slice(&passives_data)
                .with_context(|| format!("passives of '{}'", character_data.character_id))?;
            Some(rmp_serde::to_vec(&PassivesTreeAscensionData {
                ascended_nodes: old_passives.ascended_nodes,
                socketed_nodes: old_passives
                    .socketed_nodes
                    .into_iter()
                    .map(|(node, item)| migrate_item(item, items_store).map(|item| (node, item)))
                    .collect::<anyhow::Result<_>>()?,
            })?)
        } else {
            None
        };

        let inventory_data = rmp_serde::to_vec(&inventory)?;

        sqlx::query!(
            r#"
            UPDATE characters_data
            SET inventory_data = $2,
                passives_data = $3,
                data_version = $4,
                updated_at = CURRENT_TIMESTAMP
            WHERE character_id = $1
            "#,
            character_data.character_id,
            inventory_data,
            passives_data,
            TARGET_DATA_VERSION,
        )
        .execute(&mut **executor)
        .await?;
    }

    Ok(())
}

async fn migrate_stash_and_market_items(
    executor: &mut Transaction<'static, Database>,
    items_store: &ItemsStore,
) -> anyhow::Result<()> {
    let stash_items = sqlx::query!(
        r#"
        SELECT
            stash_item_id,
            item_data as "item_data: JsonValue"
        FROM stash_items
        WHERE data_version < $1
        "#,
        TARGET_DATA_VERSION
    )
    .fetch_all(&mut **executor)
    .await?;

    for stash_item in stash_items {
        let old_item: OldItemModifiers = serde_json::from_value(stash_item.item_data)
            .with_context(|| format!("stash item '{}'", stash_item.stash_item_id))?;
        let item = migrate_item(old_item, items_store)?;
        let item_data = serde_json::to_value(item)?;

        sqlx::query!(
            "UPDATE stash_items SET item_data = $1, data_version = $2 WHERE stash_item_id = $3",
            item_data,
            TARGET_DATA_VERSION,
            stash_item.stash_item_id,
        )
        .execute(&mut **executor)
        .await?;
    }

    Ok(())
}

fn migrate_item(old: OldItemModifiers, items_store: &ItemsStore) -> anyhow::Result<ItemModifiers> {
    let base = items_store
        .items
        .get(&old.base_item_id)
        .with_context(|| format!("missing item base '{}'", old.base_item_id))?;

    // Convert soul affix
    let mut old_affixes: Vec<IntermediateItemAffix> =
        old.affixes.into_iter().map(Into::into).collect();
    if let Some(affix) = old_affixes
        .iter_mut()
        .filter(|affix| matches!(affix.affix_type, AffixType::Prefix | AffixType::Suffix))
        .nth(4)
    {
        affix.affix_type = AffixType::Soul;
    }

    // Transform to normalized, source-aware affix rolls.
    let mut affixes = Vec::new();
    for old_affix in &old_affixes {
        let Some((source_base_item_id, affix_id, effect_blueprint, applies_quality)) =
            find_blueprint(old_affix, &old.base_item_id, base, items_store, &affixes)
        else {
            tracing::warn!(
                base_item_id = old.base_item_id,
                affix_name = old_affix.name,
                affix_tier = old_affix.tier,
                "removing persisted affix with no matching blueprint"
            );
            continue;
        };

        affixes.push(AffixRoll {
            affix_type: if base.categories.contains(&ItemCategory::Soul)
                && source_base_item_id.is_some()
            {
                AffixType::Soul
            } else if applies_quality && source_base_item_id.is_some() {
                AffixType::Base
            } else if source_base_item_id.is_some() {
                AffixType::Unique
            } else {
                old_affix.affix_type
            },
            base_item_id: source_base_item_id,
            affix_id,
            value: normalized_roll(
                old_affix.effects.first().map(|effect| &effect.stat_effect),
                effect_blueprint,
                old.quality,
                applies_quality,
            ),
        });
    }

    for (effect_id, effect_bp) in base.affixes.iter() {
        if !affixes.iter().any(|affix: &AffixRoll| {
            affix.base_item_id.as_deref() == Some(old.base_item_id.as_str())
                && affix.affix_id == *effect_id
        }) {
            tracing::warn!(
                base_item_id = old.base_item_id,
                affix_id = effect_id,
                "adding missing base affix with a new random roll"
            );
            affixes.push(AffixRoll {
                affix_type: if base.categories.contains(&ItemCategory::Soul) {
                    AffixType::Soul
                } else if effect_bp.is_unique {
                    AffixType::Unique
                } else {
                    AffixType::Base
                },
                base_item_id: Some(old.base_item_id.clone()),
                affix_id: effect_id.clone(),
                value: random_normalized_roll(),
            });
        }
    }

    Ok(ItemModifiers {
        base_item_id: old.base_item_id,
        name: old.name,
        rarity: old.rarity,
        level: old.level,
        quality: old.quality,
        upgrade_level: old.upgrade_level,
        affixes,
    })
}

fn find_blueprint<'a>(
    old_affix: &IntermediateItemAffix,
    base_item_id: &str,
    base: &'a ItemBase,
    items_store: &'a ItemsStore,
    used_affixes: &[AffixRoll],
) -> Option<(Option<String>, String, &'a AffixEffectBlueprint, bool)> {
    let is_used = |source_base_item_id: Option<&str>, id: &str| {
        used_affixes.iter().any(|affix| {
            affix.base_item_id.as_deref() == source_base_item_id && affix.affix_id == id
        })
    };
    let old_effect = old_affix.effects.first();

    if (old_affix.name == base.name && old_affix.tier == 0) || (old_affix.name == "Unique") {
        let candidate = base
            .affixes
            .iter()
            .filter(|(effect_id, _)| !is_used(Some(base_item_id), effect_id))
            .find(|(_, blueprint)| effect_matches(old_effect, &blueprint.effect));
        if let Some((effect_id, blueprint)) = candidate {
            return Some((
                Some(base_item_id.to_string()),
                effect_id.clone(),
                &blueprint.effect,
                !blueprint.is_unique,
            ));
        }
    }

    items_store
        .affixes_table
        .iter()
        .filter(|(id, blueprint)| {
            !is_used(None, id)
                && blueprint.name == old_affix.name
                && blueprint.tier == old_affix.tier
                && blueprint
                    .restrictions
                    .as_ref()
                    .is_none_or(|restrictions| !restrictions.is_disjoint(&base.categories))
        })
        .max_by_key(|(_, blueprint)| {
            blueprint
                .effects
                .first()
                .is_some_and(|effect| effect_matches(old_effect, effect))
        })
        .and_then(|(id, blueprint)| {
            blueprint
                .effects
                .first()
                .map(|effect| (None, id.clone(), effect, false))
        })
}

fn effect_matches(old_effect: Option<&AffixEffect>, blueprint: &AffixEffectBlueprint) -> bool {
    old_effect.is_some_and(|old| {
        old.scope == blueprint.scope
            && old.stat_effect.stat == blueprint.stat
            && old.stat_effect.modifier == blueprint.modifier
    })
}

fn normalized_roll(
    old_effect: Option<&StatEffect>,
    blueprint: &AffixEffectBlueprint,
    quality: f32,
    applies_quality: bool,
) -> u8 {
    let Some(old_effect) = old_effect else {
        return random_normalized_roll();
    };
    let quality_factor = if applies_quality && quality.is_finite() {
        1.0 + f64::from(quality.max(0.0)) * 0.01
    } else {
        1.0
    };
    let value_without_quality = old_effect.value / quality_factor;
    let range = blueprint.value.max - blueprint.value.min;
    if !value_without_quality.is_finite() || !range.is_finite() {
        return random_normalized_roll();
    }
    if range == 0.0 {
        return 50;
    }

    (((value_without_quality - blueprint.value.min) / range * 100.0)
        .round()
        .clamp(0.0, 100.0)) as u8
}

fn random_normalized_roll() -> u8 {
    rng::random_range(0..=100).unwrap_or(50) as u8
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
struct OldInventoryData {
    equipped: HashMap<ItemSlot, OldItemModifiers>,
    bag: Vec<OldItemModifiers>,
    max_bag_size: u8,
    #[serde(default)]
    sheathed: HashSet<ItemSlot>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
struct OldPassivesTreeAscensionData {
    ascended_nodes: HashMap<PassiveNodeId, u8>,
    #[serde(default)]
    socketed_nodes: HashMap<PassiveNodeId, OldItemModifiers>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct OldItemModifiers {
    base_item_id: String,
    name: String,
    rarity: ItemRarity,
    level: AreaLevel,
    affixes: Vec<OldItemAffix>,
    #[serde(default)]
    quality: f32,
    #[serde(default)]
    upgrade_level: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct OldItemAffix {
    name: String,
    family: String,
    tags: BTreeSet<AffixTag>,
    affix_type: AffixType,
    tier: u8,
    effects: Vec<OldAffixEffect>,
    #[serde(default)]
    item_level: AreaLevel,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct IntermediateItemAffix {
    name: String,
    family: String,
    tags: BTreeSet<AffixTag>,
    affix_type: AffixType,
    tier: u8,
    effects: Vec<AffixEffect>,
    #[serde(default)]
    item_level: AreaLevel,
}

impl From<OldItemAffix> for IntermediateItemAffix {
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
struct OldAffixEffect {
    scope: AffixEffectScope,
    stat_effect: OldStatEffect,
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
struct OldStatEffect {
    stat: OldStatType,
    modifier: Modifier,
    value: f64,
    #[serde(default)]
    bypass_ignore: bool,
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
enum OldStatType {
    Description(String),
    MaxPlayerLevel,
    AllocatePassive(PassiveNodeId),
    RemoveSlotUse(ItemSlot),
    GemsFind,
    ItemRarity,
    ItemAreaChance,
    ItemLevel,
    SkillLevel(#[serde(default)] StatSkillFilter),
    Armor(Option<DamageStatType>),
    DamageTaken {
        #[serde(default)]
        skill_type: Option<SkillType>,
        #[serde(default)]
        damage_type: Option<DamageType>,
    },
    CritDamageTaken {
        #[serde(default)]
        skill_type: Option<SkillType>,
    },
    Block(#[serde(default)] Option<SkillType>),
    BlockDamageTaken,
    Evade(#[serde(default)] Option<DamageType>),
    EvadeDamageTaken,
    StatusResistance {
        #[serde(default)]
        status_id: Option<StatusId>,
        #[serde(default)]
        skill_type: Option<SkillType>,
    },
    Damage {
        #[serde(flatten)]
        skill_filter: StatSkillFilter,
        #[serde(default)]
        damage_type: Option<DamageType>,
        #[serde(default)]
        min_max: Option<MinMax>,
        #[serde(default)]
        is_hit: Option<bool>,
    },
    CritChance(#[serde(default)] StatSkillFilter),
    CritDamage(#[serde(default)] StatSkillFilter),
    StatusPower {
        #[serde(flatten)]
        status_filter: StatStatusFilter,
        #[serde(flatten)]
        skill_filter: StatSkillFilter,
        #[serde(default)]
        min_max: Option<MinMax>,
    },
    StatusDuration {
        #[serde(flatten)]
        status_filter: StatStatusFilter,
        #[serde(flatten)]
        skill_filter: StatSkillFilter,
    },
    StatusStacks {
        #[serde(flatten)]
        status_filter: StatStatusFilter,
        #[serde(flatten)]
        skill_filter: StatSkillFilter,
    },
    StatusEscalation {
        #[serde(flatten)]
        status_filter: StatStatusFilter,
        #[serde(flatten)]
        skill_filter: StatSkillFilter,
    },
    StatusFaster {
        #[serde(flatten)]
        status_filter: StatStatusFilter,
        #[serde(flatten)]
        skill_filter: StatSkillFilter,
    },
    SuccessChance {
        #[serde(flatten)]
        skill_filter: StatSkillFilter,
        #[serde(default)]
        effect_type: Option<StatSkillEffectType>,
    },
    Speed(#[serde(default)] StatSkillFilter),
    RestoreOnHit {
        restore_type: RestoreType,
        #[serde(default)]
        skill_type: Option<SkillType>,
    },
    Restore {
        #[serde(default)]
        restore_type: Option<RestoreType>,
        #[serde(flatten)]
        skill_filter: StatSkillFilter,
        #[serde(default)]
        modifier_description: RestoreModifier,
    },
    Life,
    LifeRegen,
    Mana,
    ManaRegen,
    ManaCost {
        #[serde(flatten)]
        skill_filter: StatSkillFilter,
    },
    TakeFromManaBeforeLife,
    TakeFromLifeBeforeMana,
    MovementSpeed,
    ThreatGain,
    Lucky {
        #[serde(flatten)]
        skill_filter: StatSkillFilter,
        roll_type: LuckyRollType,
    },
    RefreshedCooldown {
        #[serde(flatten)]
        skill_filter: StatSkillFilter,
    },
    SkillConditionalModifier {
        stat: Box<OldStatType>,
        #[serde(flatten)]
        skill_filter: StatSkillFilter,
        #[serde(default)]
        conditions: Vec<Condition>,
    },
    StatConditionalModifier {
        stat: Box<OldStatType>,
        conditions: Vec<Condition>,
        #[serde(default)]
        conditions_duration: u32,
    },
    StatConverter(OldStatConverterSpecs),
    SkillTargetModifier {
        #[serde(flatten)]
        skill_filter: StatSkillFilter,
        #[serde(default)]
        range: Option<SkillRange>,
        #[serde(default)]
        shape: Option<SkillShape>,
        #[serde(default)]
        repeat: Option<StatSkillRepeat>,
    },
    SkillRepeat {
        #[serde(flatten)]
        skill_filter: StatSkillFilter,
    },
    SkillEffectModifier {
        #[serde(flatten)]
        skill_filter: StatSkillFilter,
        #[serde(default)]
        unblockable: Option<bool>,
        #[serde(default)]
        avoidable: Option<bool>,
    },
    TriggerEffectModifier {
        stat: Box<OldStatType>,
        source: TriggerEffectModifierSource,
        #[serde(flatten)]
        skill_filter: StatSkillFilter,
    },
    GoldFind,
    PowerLevel,
    Description2(String),
}

impl From<OldStatType> for StatType {
    fn from(value: OldStatType) -> Self {
        match value {
            OldStatType::Description(value) => Self::Description(value),
            OldStatType::MaxPlayerLevel => Self::MaxPlayerLevel,
            OldStatType::AllocatePassive(value) => Self::AllocatePassive(value),
            OldStatType::RemoveSlotUse(value) => Self::RemoveSlotUse(value),
            OldStatType::GemsFind => Self::GemsFind,
            OldStatType::ItemRarity => Self::ItemRarity,
            OldStatType::ItemAreaChance => Self::ItemAreaChance,
            OldStatType::ItemLevel => Self::ItemLevel,
            OldStatType::SkillLevel(value) => Self::SkillLevel(value),
            OldStatType::Armor(value) => Self::Armor(value),
            OldStatType::DamageTaken {
                skill_type,
                damage_type,
            } => Self::DamageTaken {
                skill_type,
                damage_type,
            },
            OldStatType::CritDamageTaken { skill_type } => Self::CritDamageTaken { skill_type },
            OldStatType::Block(value) => Self::Block(value),
            OldStatType::BlockDamageTaken => Self::BlockDamageTaken,
            OldStatType::Evade(value) => Self::Evade(value),
            OldStatType::EvadeDamageTaken => Self::EvadeDamageTaken,
            OldStatType::StatusResistance {
                status_id,
                skill_type,
            } => Self::StatusResistance {
                status_id,
                skill_type,
            },
            OldStatType::Damage {
                skill_filter,
                damage_type,
                min_max,
                is_hit,
            } => Self::Damage {
                skill_filter,
                damage_type: damage_type.map(Into::into),
                min_max,
                is_hit,
            },
            OldStatType::CritChance(value) => Self::CritChance(value),
            OldStatType::CritDamage(value) => Self::CritDamage(value),
            OldStatType::StatusPower {
                status_filter,
                skill_filter,
                min_max,
            } => Self::StatusPower {
                status_filter,
                skill_filter,
                min_max,
            },
            OldStatType::StatusDuration {
                status_filter,
                skill_filter,
            } => Self::StatusDuration {
                status_filter,
                skill_filter,
            },
            OldStatType::StatusStacks {
                status_filter,
                skill_filter,
            } => Self::StatusStacks {
                status_filter,
                skill_filter,
            },
            OldStatType::StatusEscalation {
                status_filter,
                skill_filter,
            } => Self::StatusEscalation {
                status_filter,
                skill_filter,
            },
            OldStatType::StatusFaster {
                status_filter,
                skill_filter,
            } => Self::StatusFaster {
                status_filter,
                skill_filter,
            },
            OldStatType::SuccessChance {
                skill_filter,
                effect_type,
            } => Self::SuccessChance {
                skill_filter,
                effect_type,
            },
            OldStatType::Speed(value) => Self::Speed(value),
            OldStatType::RestoreOnHit {
                restore_type,
                skill_type,
            } => Self::RestoreOnHit {
                restore_type,
                skill_type,
            },
            OldStatType::Restore {
                restore_type,
                skill_filter,
                modifier_description,
            } => Self::Restore {
                restore_type,
                skill_filter,
                modifier_description,
            },
            OldStatType::Life => Self::Life,
            OldStatType::LifeRegen => Self::LifeRegen,
            OldStatType::Mana => Self::Mana,
            OldStatType::ManaRegen => Self::ManaRegen,
            OldStatType::ManaCost { skill_filter } => Self::ManaCost { skill_filter },
            OldStatType::TakeFromManaBeforeLife => Self::TakeFromManaBeforeLife,
            OldStatType::TakeFromLifeBeforeMana => Self::TakeFromLifeBeforeMana,
            OldStatType::MovementSpeed => Self::MovementSpeed,
            OldStatType::ThreatGain => Self::ThreatGain,
            OldStatType::Lucky {
                skill_filter,
                roll_type,
            } => Self::Lucky {
                skill_filter,
                roll_type,
            },
            OldStatType::RefreshedCooldown { skill_filter } => {
                Self::RefreshedCooldown { skill_filter }
            }
            OldStatType::SkillConditionalModifier {
                stat,
                skill_filter,
                conditions,
            } => Self::SkillConditionalModifier {
                stat: Box::new((*stat).into()),
                skill_filter,
                conditions,
            },
            OldStatType::StatConditionalModifier {
                stat,
                conditions,
                conditions_duration,
            } => Self::StatConditionalModifier {
                stat: Box::new((*stat).into()),
                conditions,
                conditions_duration,
            },
            OldStatType::StatConverter(value) => Self::StatConverter(value.into()),
            OldStatType::SkillTargetModifier {
                skill_filter,
                range,
                shape,
                repeat,
            } => Self::SkillTargetModifier {
                skill_filter,
                range,
                shape,
                repeat,
            },
            OldStatType::SkillRepeat { skill_filter } => Self::SkillRepeat { skill_filter },
            OldStatType::SkillEffectModifier {
                skill_filter,
                unblockable,
                avoidable,
            } => Self::SkillEffectModifier {
                skill_filter,
                unblockable,
                avoidable,
            },
            OldStatType::TriggerEffectModifier {
                stat,
                source,
                skill_filter,
            } => Self::TriggerEffectModifier {
                stat: Box::new((*stat).into()),
                source,
                skill_filter,
            },
            OldStatType::GoldFind => Self::GoldFind,
            OldStatType::PowerLevel => Self::PowerLevel,
            OldStatType::Description2(value) => Self::Description2(value),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct OldStatConverterSpecs {
    source: StatConverterSource,
    stat: Box<OldStatType>,
    #[serde(default)]
    is_extra: bool,
    #[serde(default)]
    skill_type: Option<SkillType>,
    #[serde(default)]
    uncapped: bool,
}

impl From<OldStatConverterSpecs> for StatConverterSpecs {
    fn from(value: OldStatConverterSpecs) -> Self {
        Self {
            source: value.source,
            stat: Box::new((*value.stat).into()),
            is_extra: value.is_extra,
            skill_filter: StatSkillFilter {
                skill_type: value.skill_type,
                skill_id: None,
            },
            uncapped: value.uncapped,
        }
    }
}
