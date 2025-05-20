use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

pub use super::skill::{Range, Shape};
use super::{
    item_affix::{AffixEffectBlueprint, AffixRestriction, EffectsMap, ItemAffix},
    skill::DamageType,
    world::AreaLevel,
};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Default)]
pub enum ItemRarity {
    #[default]
    Normal,
    Magic,
    Rare,
    Unique,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ItemSlot {
    Amulet,
    Body,
    Boots,
    Gloves,
    Helmet,
    Ring,
    Shield,
    Trinket,
    Weapon,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ItemBase {
    pub name: String,
    pub icon: String,
    #[serde(default)]
    pub description: Option<String>,

    pub slot: ItemSlot,
    #[serde(default)]
    pub extra_slots: HashSet<ItemSlot>,
    pub affix_restrictions: HashSet<AffixRestriction>,

    #[serde(default)]
    pub min_area_level: Option<AreaLevel>,
    #[serde(default)]
    pub rarity: ItemRarity,
    #[serde(default)]
    pub affixes: Vec<AffixEffectBlueprint>,

    // TODO:
    // #[serde(default)]
    // pub skills: Vec<BaseSkillSpecs>,s
    #[serde(default)]
    pub weapon_specs: Option<WeaponSpecs>,
    #[serde(default)]
    pub armor_specs: Option<ArmorSpecs>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ItemSpecs {
    pub name: String,

    pub base: ItemBase,
    pub rarity: ItemRarity,
    pub level: AreaLevel,

    // TODO
    // pub skills: Vec<BaseSkillSpecs>,
    pub weapon_specs: Option<WeaponSpecs>,
    pub armor_specs: Option<ArmorSpecs>,

    pub affixes: Vec<ItemAffix>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct WeaponSpecs {
    #[serde(default)]
    pub range: Range,
    #[serde(default)]
    pub shape: Shape,

    pub cooldown: f32,

    // #[serde(rename_all = "snake_case")]
    pub damage: HashMap<DamageType, (f64, f64)>,

    pub crit_chances: f32,
    pub crit_damage: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct ArmorSpecs {
    pub armor: f64,
    // TODO: Block chances
}

impl ItemSpecs {
    pub fn aggregate_effects(&self) -> EffectsMap {
        self.affixes
            .iter()
            .flat_map(|affix| affix.effects.iter())
            .fold(EffectsMap(HashMap::new()), |mut effects_map, effect| {
                *effects_map
                    .0
                    .entry((effect.stat, effect.modifier))
                    .or_default() += effect.value;
                effects_map
            })
    }
}
