use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

use crate::{
    data::{
        chance::ChanceRange,
        item::BaseItemId,
        modifier::Modifier,
        stat_effect::{DamageStatType, EffectsMap},
    },
    serde_utils::is_false,
};

use super::{
    area::AreaLevel,
    item::ItemCategory,
    stat_effect::{StatEffect, StatType},
};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum AffixType {
    Base,
    Prefix,
    Suffix,
    Soul,
    Unique,
    Upgrade,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum AffixTag {
    Unique,

    Speed,
    Skill,

    Armor,
    Attack,
    Spell,

    Critical,
    Stealth,
    Threat,
    Status,

    Life,
    Mana,

    Physical,
    Fire,
    Storm,
    Poison,

    Gold,
    // TODO: add others
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AffixEffectScope {
    #[default]
    Local,
    Global,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ItemAffixBlueprint {
    pub name: String,
    pub family: String, // Cannot generate multiple affixes of same category on item
    pub tags: BTreeSet<AffixTag>,

    pub affix_type: AffixType,
    pub tier: u8,
    pub weight: u64, // Bigger weight means more chance to have affix

    #[serde(default)]
    pub restrictions: Option<BTreeSet<ItemCategory>>,
    pub item_level: AreaLevel,

    #[serde(default)]
    pub effects: Vec<AffixEffectBlueprint>,
    // #[serde(default)]
    // pub triggers: Vec<TriggeredEffect>, // TODO
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct AffixEffectBlueprint {
    pub stat: StatType,
    pub modifier: Modifier,
    pub scope: AffixEffectScope,
    pub value: ChanceRange<f64>,

    #[serde(default, skip_serializing_if = "is_false")]
    pub bypass_ignore: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ItemAffix {
    pub base_item_id: Option<BaseItemId>,
    pub affix_id: String,

    pub item_level: AreaLevel,
    pub affix_type: AffixType,
    pub effects: Vec<AffixEffect>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct AffixEffect {
    pub scope: AffixEffectScope,
    pub stat_effect: StatEffect,
}

pub fn aggregate_effects(
    affixes: &[ItemAffix],
    scope: AffixEffectScope,
    split_elements: bool,
) -> EffectsMap {
    affixes
        .iter()
        .flat_map(|affix| affix.effects.iter())
        .filter(|e| e.scope == scope)
        .fold(EffectsMap::default(), |mut effects_map, effect| {
            if split_elements
                && matches!(
                    &effect.stat_effect.stat,
                    StatType::Armor(Some(DamageStatType::Elemental))
                )
            {
                for stat in [
                    StatType::Armor(Some(DamageStatType::Fire)),
                    StatType::Armor(Some(DamageStatType::Poison)),
                    StatType::Armor(Some(DamageStatType::Storm)),
                ] {
                    *effects_map
                        .0
                        .entry((
                            stat,
                            effect.stat_effect.modifier,
                            effect.stat_effect.bypass_ignore,
                        ))
                        .or_default() += effect.stat_effect.value;
                }
            } else {
                *effects_map
                    .0
                    .entry((
                        effect.stat_effect.stat.clone(),
                        effect.stat_effect.modifier,
                        effect.stat_effect.bypass_ignore,
                    ))
                    .or_default() += effect.stat_effect.value;
            }

            effects_map
        })
}
