use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::data::{
    chance::{Chance, ChanceRange},
    stat_effect::{Modifier, StatType},
    trigger::TriggerSpecs,
};

pub use super::stat_effect::DamageType;
use super::{
    character_status::StatusSpecs, item::ItemSlot, passive::StatEffect, stat_effect::DamageMap,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BaseSkillSpecs {
    pub name: String,
    pub icon: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub skill_type: SkillType,

    pub cooldown: f32,
    #[serde(default)]
    pub mana_cost: f64,

    #[serde(default)]
    pub upgrade_cost: f64,
    #[serde(default)]
    pub upgrade_effects: Vec<StatEffect>,

    #[serde(default)]
    pub modifier_effects: Vec<ModifierEffect>,

    #[serde(default)]
    pub targets: Vec<SkillTargetsGroup>,
    #[serde(default)]
    pub triggers: Vec<TriggerSpecs>,
    // TODO: special upgrades at some levels?
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SkillSpecs {
    pub base: BaseSkillSpecs,

    pub cooldown: f32,
    pub mana_cost: f64,

    pub upgrade_level: u16,
    pub next_upgrade_cost: f64,

    pub targets: Vec<SkillTargetsGroup>,
    pub triggers: Vec<TriggerSpecs>,

    pub item_slot: Option<ItemSlot>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SkillState {
    pub elapsed_cooldown: f32,

    pub is_ready: bool,
    pub just_triggered: bool,
}

#[derive(
    Serialize, Deserialize, Debug, Clone, Copy, Default, PartialEq, Eq, Hash, PartialOrd, Ord,
)]
pub enum SkillType {
    #[default]
    Attack,
    Spell,
}

impl SkillType {
    pub fn iter() -> impl Iterator<Item = SkillType> {
        [SkillType::Attack, SkillType::Spell].into_iter()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ModifierEffect {
    pub effects: Vec<StatEffect>,
    pub source: ModifierEffectSource,
    pub factor: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum ModifierEffectSource {
    ItemStats {
        slot: Option<ItemSlot>,
        item_stats: ItemStatsSource,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum ItemStatsSource {
    Damage(Option<DamageType>), // TODO: split in min and max
    Armor,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SkillTargetsGroup {
    #[serde(default)]
    pub range: SkillRange,
    #[serde(default)]
    pub target_type: TargetType,
    #[serde(default)]
    pub shape: SkillShape,
    #[serde(default)]
    pub target_dead: bool,
    #[serde(default)]
    pub repeat: SkillRepeat,

    pub effects: Vec<SkillEffect>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SkillRepeat {
    pub value: ChanceRange<u8>,
    pub target: SkillRepeatTarget,
}

impl Default for SkillRepeat {
    fn default() -> Self {
        Self {
            value: ChanceRange {
                min: 1,
                max: 1,
                lucky_chance: 0.0,
            },
            target: SkillRepeatTarget::Any,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum SkillRepeatTarget {
    Any,
    Same,
    Different,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SkillEffect {
    #[serde(default)]
    pub failure_chance: Chance,

    #[serde(flatten)]
    pub effect_type: SkillEffectType,

    #[serde(default)]
    pub ignore_stat_effects: HashSet<StatType>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum SkillEffectType {
    FlatDamage {
        damage: DamageMap,
        #[serde(default)]
        crit_chance: Chance,
        #[serde(default)]
        crit_damage: f64,
    },
    ApplyStatus {
        statuses: Vec<ApplyStatusEffect>,
        duration: ChanceRange<f64>,
    },
    Restore {
        restore_type: RestoreType,
        value: ChanceRange<f64>,
        modifier: Modifier,
    },
    Resurrect,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ApplyStatusEffect {
    pub status_type: StatusSpecs,
    #[serde(default)]
    pub value: ChanceRange<f64>,
    #[serde(default)]
    pub cumulate: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum TargetType {
    #[default]
    Enemy,
    Friend,
    Me,
}

#[derive(
    Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Default, Eq, Hash, PartialOrd, Ord,
)]
pub enum SkillRange {
    #[default]
    Melee,
    Distance,
    Any,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Default)]
pub enum SkillShape {
    #[default]
    Single,
    Vertical2,
    Horizontal2,
    Horizontal3,
    Square4,
    All,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum RestoreType {
    Life,
    Mana,
}
