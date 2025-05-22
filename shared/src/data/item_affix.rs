use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use super::world::AreaLevel;

pub use super::effect::{DamageType, EffectModifier, StatEffect, StatType};
pub use super::skill::{Range, Shape};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct EffectsMap(pub HashMap<(StatType, EffectModifier), f64>);

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum AffixType {
    Prefix,
    Suffix,
    Unique,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
// TODO: add others
pub enum AffixRestriction {
    // Major type:
    Armor,
    AttackWeapon,
    SpellWeapon,
    MeleeWeapon,
    RangedWeapon,
    Shield,
    Focus,
    Jewelry,
    Trinket,
    // Sub type:
    Body,
    Boots,
    Cloak,
    Gloves,
    Helmet,
    Ring,
    Amulet,
    Relic,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Eq, PartialEq, Hash)]
// TODO: add others
pub enum AffixTag {
    Attack,
    Armor,
    Critical,
    Fire,
    Gold,
    Life,
    Mana,
    Physical,
    Poison,
    Speed,
    Spell,
}

// #[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
// pub enum AffixEffectScope {
//     Local,
//     Global,
// }

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ItemAffixBlueprint {
    pub name: String,
    pub family: String, // Cannot generate multiple affixes of same category on item
    pub tags: HashSet<AffixTag>,

    pub affix_type: AffixType,
    pub tier: u8,
    pub weight: u64, // Bigger weight means more chances to have affix

    #[serde(default)]
    pub restrictions: Option<HashSet<AffixRestriction>>,
    pub item_level: AreaLevel,

    pub effects: Vec<AffixEffectBlueprint>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct AffixEffectBlueprint {
    pub stat: StatType,
    pub modifier: EffectModifier,
    // pub scope: AffixEffectScope,
    pub min: f64,
    pub max: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ItemAffix {
    pub name: String,
    pub family: String,
    pub tags: HashSet<AffixTag>,

    pub affix_type: AffixType,
    pub tier: u8,

    pub effects: Vec<StatEffect>,
}

impl From<&EffectsMap> for Vec<StatEffect> {
    fn from(val: &EffectsMap) -> Self {
        val.0
            .iter()
            .map(|((stat, effect_type), value)| StatEffect {
                stat: *stat,
                modifier: *effect_type,
                value: *value,
            })
            .collect()
    }
}

impl EffectsMap {
    pub fn combine_all(maps: impl Iterator<Item = EffectsMap>) -> Self {
        let mut result: HashMap<(StatType, EffectModifier), f64> = HashMap::new();

        for map in maps {
            for ((target, modifier), value) in map.0 {
                let entry = result.entry((target, modifier)).or_insert(match modifier {
                    EffectModifier::Flat => 0.0,
                    EffectModifier::Multiplier => 1.0,
                });

                match modifier {
                    EffectModifier::Flat => *entry += value,
                    EffectModifier::Multiplier => *entry *= 1.0 + value,
                }
            }
        }

        for ((_, modifier), val) in result.iter_mut() {
            if *modifier == EffectModifier::Multiplier {
                *val -= 1.0;
            }
        }

        EffectsMap(result)
    }
}
