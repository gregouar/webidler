use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::skill::SkillType;

#[derive(
    Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default,
)]
pub enum DamageType {
    #[default]
    Physical,
    Fire,
    Poison,
}

impl DamageType {
    pub fn iter() -> impl Iterator<Item = DamageType> {
        [DamageType::Physical, DamageType::Fire, DamageType::Poison].into_iter()
    }
}

pub type DamageMap = HashMap<DamageType, (f64, f64)>;

#[derive(
    Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default,
)]
pub enum Modifier {
    #[default]
    Multiplier,
    Flat,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum StatType {
    Life,
    LifeRegen,
    Mana,
    ManaRegen,
    Armor(DamageType),
    Block,
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
    SpellPower,
    CritChances(#[serde(default)] Option<SkillType>),
    CritDamage(#[serde(default)] Option<SkillType>),
    Speed(#[serde(default)] Option<SkillType>),
    MovementSpeed,
    GoldFind,
    // TODO: ReducedManaCost?
    // TODO: TriggerSkill (effect trigger + Box Skill) => separate because cannot be hashed/copy etc
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct StatEffect {
    pub stat: StatType,
    pub modifier: Modifier,
    pub value: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct EffectsMap(pub HashMap<(StatType, Modifier), f64>);

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
        EffectsMap(maps.flat_map(|m| m.0.into_iter()).fold(
            HashMap::new(),
            |mut result, ((target, modifier), value)| {
                result
                    .entry((target, modifier))
                    .and_modify(|entry| match modifier {
                        Modifier::Flat => *entry += value,
                        Modifier::Multiplier => *entry = (*entry + 1.0) * (1.0 + value) - 1.0,
                    })
                    .or_insert(value);
                result
            },
        ))
    }
}
