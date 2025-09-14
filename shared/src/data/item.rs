use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

use crate::data::{item_affix::AffixType, trigger::TriggerSpecs};

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
    Masterwork,
    Unique,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter)]
pub enum ItemSlot {
    Accessory,
    Helmet,
    Amulet,
    Weapon,
    Body,
    Shield,
    Gloves,
    Boots,
    Ring,
}

impl TryFrom<usize> for ItemSlot {
    type Error = anyhow::Error;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        use ItemSlot::*;
        Ok(match value {
            0 => Accessory,
            1 => Amulet,
            2 => Body,
            3 => Boots,
            4 => Gloves,
            5 => Helmet,
            6 => Ring,
            7 => Shield,
            8 => Weapon,
            _ => return Err(anyhow::anyhow!("invalid slot")),
        })
    }
}

impl From<ItemSlot> for usize {
    fn from(value: ItemSlot) -> Self {
        use ItemSlot::*;
        match value {
            Accessory => 0,
            Amulet => 1,
            Body => 2,
            Boots => 3,
            Gloves => 4,
            Helmet => 5,
            Ring => 6,
            Shield => 7,
            Weapon => 8,
        }
    }
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
pub struct ItemModifiers {
    pub base_item_id: String,
    pub name: String,

    pub rarity: ItemRarity,
    pub level: AreaLevel,

    pub affixes: Vec<ItemAffix>,

    #[serde(default)]
    pub quality: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ItemSpecs {
    pub base: ItemBase,
    pub modifiers: ItemModifiers,

    pub weapon_specs: Option<WeaponSpecs>,
    pub armor_specs: Option<ArmorSpecs>,

    // To indicate it comes from old game and not dropped during current one
    pub old_game: bool,
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

impl ItemModifiers {
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

    pub fn count_affixes(&self, affix_type: AffixType) -> usize {
        self.affixes
            .iter()
            .filter(|affix| affix_type == affix.affix_type)
            .count()
    }

    pub fn count_nonunique_affixes(&self) -> usize {
        self.affixes
            .iter()
            .filter(|affix| affix.affix_type != AffixType::Unique)
            .count()
    }

    pub fn get_families(&self) -> HashSet<String> {
        self.affixes
            .iter()
            .map(|affix| affix.family.clone())
            .collect()
    }
}
