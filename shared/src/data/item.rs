use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

use crate::data::trigger::TriggerSpecs;

pub use super::skill::{SkillRange, SkillShape};
use super::{
    area::AreaLevel,
    item_affix::{AffixEffectBlueprint, AffixEffectScope, ItemAffix},
    stat_effect::{DamageMap, EffectsMap},
};

#[derive(
    Serialize,
    Deserialize,
    Debug,
    Clone,
    Copy,
    PartialEq,
    PartialOrd,
    Eq,
    Ord,
    Default,
    Hash,
    EnumIter,
)]
pub enum ItemRarity {
    #[default]
    Normal,
    Magic,
    Rare,
    Unique,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter)]
pub enum ItemSlot {
    Accessory,
    Amulet,
    Body,
    Boots,
    Gloves,
    Helmet,
    Ring,
    Shield,
    Weapon,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter)]
// TODO: add others
pub enum ItemCategory {
    // Major categories:
    Armor,
    AttackWeapon,
    SpellWeapon,
    MeleeWeapon,
    MeleeWeapon1H,
    MeleeWeapon2H,
    RangedWeapon,
    Shield,
    Focus,
    Jewelry,
    Accessory,
    // Minor categories:
    Amulet,
    Body,
    Boots,
    Cloak,
    Gloves,
    Helmet,
    Relic,
    Ring,
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
    pub categories: HashSet<ItemCategory>,

    #[serde(default)]
    pub min_area_level: AreaLevel,
    #[serde(default)]
    pub rarity: ItemRarity,
    #[serde(default)]
    pub affixes: Vec<AffixEffectBlueprint>,
    #[serde(default)]
    pub triggers: Vec<TriggerSpecs>,

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

    pub weapon_specs: Option<WeaponSpecs>,
    pub armor_specs: Option<ArmorSpecs>,

    pub affixes: Vec<ItemAffix>,
    pub triggers: Vec<TriggerSpecs>,

    #[serde(default)] // TODO: Remove later, only for save backward comp
    pub old_game: bool, // To indicate it comes from old game and not dropped during current one
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct WeaponSpecs {
    #[serde(default)]
    pub range: SkillRange,
    #[serde(default)]
    pub shape: SkillShape,

    pub cooldown: f32,

    // #[serde(rename_all = "snake_case")]
    pub damage: DamageMap,

    pub crit_chances: f32,
    pub crit_damage: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct ArmorSpecs {
    #[serde(default)]
    pub armor: f64,
    #[serde(default)]
    pub block: f32,
}

impl ItemSpecs {
    pub fn aggregate_effects(&self, scope: AffixEffectScope) -> EffectsMap {
        self.affixes
            .iter()
            .flat_map(|affix| affix.effects.iter())
            .filter(|e| e.scope == scope)
            .fold(EffectsMap(HashMap::new()), |mut effects_map, effect| {
                *effects_map
                    .0
                    .entry((effect.stat_effect.stat, effect.stat_effect.modifier))
                    .or_default() += effect.stat_effect.value;
                effects_map
            })
    }
}
