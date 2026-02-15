use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::data::{
    chance::{Chance, ChanceRange},
    conditional_modifier::ConditionalModifier,
    modifier::{ModifiableValue, Modifier},
    stat_effect::{MinMax, StatType},
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
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SkillSpecs {
    pub base: BaseSkillSpecs,

    pub cooldown: ModifiableValue<f32>,
    pub mana_cost: ModifiableValue<f64>,

    pub upgrade_level: u16,
    pub next_upgrade_cost: f64,

    pub targets: Vec<SkillTargetsGroup>,
    pub triggers: Vec<TriggerSpecs>,

    pub item_slot: Option<ItemSlot>,

    #[serde(default)] //For retro compatibility
    pub level_modifier: u16,
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
    Curse,
    Blessing,
    Other,
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
    #[serde(default)]
    pub hidden: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum ModifierEffectSource {
    ItemStats {
        slot: Option<ItemSlot>,
        item_stats: ItemStatsSource,
    },
    PlaceHolder,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum ItemStatsSource {
    Armor,
    Cooldown,
    CritChance,
    CritDamage,
    Damage {
        #[serde(default)]
        damage_type: Option<DamageType>,
        #[serde(default)]
        min_max: Option<MinMax>,
    },
    Range,
    Shape,
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
                lucky_chance: 0.0.into(),
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
    #[serde(default = "Chance::new_sure")]
    pub success_chance: Chance,

    #[serde(flatten)]
    pub effect_type: SkillEffectType,

    #[serde(default)]
    pub ignore_stat_effects: HashSet<StatType>,

    #[serde(default)]
    pub conditional_modifiers: Vec<ConditionalModifier>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum SkillEffectType {
    FlatDamage {
        damage: DamageMap,
        #[serde(default)]
        crit_chance: Chance,
        #[serde(default)]
        crit_damage: ModifiableValue<f64>,
        #[serde(default, skip_serializing)]
        ignore_armor: bool, // TODO: Remove
    },
    ApplyStatus {
        statuses: Vec<ApplyStatusEffect>,
        duration: ChanceRange<ModifiableValue<f64>>,
    },
    Restore {
        restore_type: RestoreType,
        value: ChanceRange<ModifiableValue<f64>>,
        modifier: Modifier,
    },
    Resurrect,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ApplyStatusEffect {
    pub status_type: StatusSpecs,
    #[serde(default)]
    pub value: ChanceRange<ModifiableValue<f64>>,
    #[serde(default)]
    pub cumulate: bool,
    #[serde(default)]
    pub replace_on_value_only: bool,
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

#[derive(
    Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Default, Eq, Hash, PartialOrd, Ord,
)]
pub enum SkillShape {
    #[default]
    Single,
    Vertical2,
    Horizontal2,
    Horizontal3,
    Square4,
    All,
    Contact,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum RestoreType {
    Life,
    Mana,
}
