use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

use crate::{
    data::{
        chance::{Chance, ChanceRange},
        character::CharacterId,
        character_status::StatusId,
        conditional_modifier::{Condition, ConditionalModifier},
        item::{ItemCategory, ItemSpecs},
        modifier::ModifiableValue,
        stat_effect::{
            Matchable, MinMax, StatConverterSource, StatEffect, StatSkillFilter, StatType,
        },
        trigger::TriggerSpecs,
        values::{Cooldown, NonNegative},
    },
    serde_utils::default_1f64,
};

pub use super::stat_effect::DamageType;
use super::{item::ItemSlot, stat_effect::DamageMap};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct BaseSkillSpecs {
    pub name: String,
    pub icon: String,

    #[serde(default)]
    pub description: String,

    pub skill_type: SkillType,

    pub cooldown: NonNegative,
    #[serde(default)]
    pub mana_cost: NonNegative,
    #[serde(default)]
    pub required_item: Option<SkillRequiredItem>,

    #[serde(default)]
    pub upgrade_cost: f64,
    #[serde(default)]
    pub upgrade_effects: Vec<SkillUpgradeEffect>,

    #[serde(default)]
    pub modifier_effects: Vec<ModifierEffect>,

    #[serde(default)]
    pub targets: Vec<SkillTargetsGroup>,
    #[serde(default)]
    pub triggers: Vec<TriggerSpecs>,

    #[serde(default)]
    pub auto_use_conditions: Vec<Condition>,

    #[serde(default)]
    pub ignore_stat_effects: HashSet<StatType>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SkillUpgradeEffect {
    #[serde(flatten)]
    pub stat_effect: StatEffect,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SkillSpecs {
    pub skill_id: String,

    pub name: String,
    pub icon: String,
    pub description: String,
    pub skill_type: SkillType,

    // Should we split in two here?
    pub cooldown: ModifiableValue<NonNegative>,
    pub mana_cost: ModifiableValue<NonNegative>,

    pub targets: Vec<SkillTargetsGroup>,
    pub triggers: Vec<TriggerSpecs>,

    pub level_modifier: u16,

    #[serde(default)]
    pub ignore_stat_effects: HashSet<StatType>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SkillState {
    pub elapsed_cooldown: Cooldown,

    pub is_ready: bool,
    pub just_triggered: bool,
}

#[derive(
    Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, EnumIter,
)]
pub enum SkillType {
    Attack,
    Spell,
    Curse,
    Blessing,
    Other,
}

impl Matchable for SkillType {
    fn is_match(&self, other: &Self) -> bool {
        *self == *other
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Default)]
pub struct SkillRequiredItem {
    #[serde(default)]
    pub slot: Option<ItemSlot>,
    #[serde(default)]
    pub category: Option<ItemCategory>,
}

impl SkillRequiredItem {
    pub fn is_match(&self, item_slot: ItemSlot, item_specs: &ItemSpecs) -> bool {
        self.slot
            .map(|slot| slot == item_slot || item_specs.base.extra_slots.contains(&slot))
            .unwrap_or(true)
            && self
                .category
                .map(|category| item_specs.base.categories.contains(&category))
                .unwrap_or(true)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ModifierEffect {
    pub effects: Vec<StatEffect>,
    pub source: ModifierEffectSource,
    pub factor: f64,
    #[serde(default)]
    pub hidden: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ModifierEffectSource {
    ItemStats {
        item_stats: ItemStatsSource,
        #[serde(flatten)]
        required_item: SkillRequiredItem,
    },
    CharacterStats(StatConverterSource),
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum ItemStatsSource {
    Armor,
    Block,
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
    Equipped,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SkillRepeat {
    pub value: ChanceRange<u8>,
    pub target: SkillRepeatTarget,
    #[serde(default)]
    pub repeat_cooldown: NonNegative,
}

impl Default for SkillRepeat {
    fn default() -> Self {
        Self {
            value: ChanceRange {
                min: 1,
                max: 1,
                lucky_chance: Default::default(),
            },
            target: SkillRepeatTarget::Any,
            repeat_cooldown: Default::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
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

    #[serde(default)]
    pub independent_application: bool,
}

#[allow(clippy::large_enum_variant)]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum SkillEffectType {
    FlatDamage {
        damage: DamageMap,
        #[serde(default)]
        crit_chance: Chance,
        #[serde(default)]
        crit_damage: ModifiableValue<f64>,
        #[serde(default)]
        unblockable: bool,
    },
    ApplyStatus {
        status_id: StatusId,

        value: ChanceRange<ModifiableValue<NonNegative>>,
        #[serde(default = "default_1f64")]
        value_factor: f64, // For tooltip purposes

        // On skill definition, overwrite default status value when some
        // On skill computed specs, get derived values
        #[serde(default)]
        duration: Option<ChanceRange<ModifiableValue<NonNegative>>>,
        #[serde(default)]
        escalation: Option<ModifiableValue<NonNegative>>,
        #[serde(default)]
        max_stacks: Option<ModifiableValue<u8>>,
        #[serde(default)]
        damage_type: Option<DamageType>,
        #[serde(default)]
        avoidable: Option<bool>,

        #[serde(default)]
        replace_on_value_only: bool,
        // TODO? Computed status effects for inherit_trigger_owner?
    },
    Restore {
        restore_type: RestoreType,
        value: ChanceRange<ModifiableValue<f64>>,
        modifier: RestoreModifier,
    },
    Resurrect,
    RefreshCooldown {
        #[serde(flatten)]
        skill_filter: StatSkillFilter,
        value: ChanceRange<ModifiableValue<f64>>,
        modifier: RestoreModifier,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum RestoreModifier {
    Flat,
    Percent,
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

impl Matchable for SkillRange {
    fn is_match(&self, other: &Self) -> bool {
        *self == *other
    }
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

impl Matchable for RestoreType {
    fn is_match(&self, other: &Self) -> bool {
        *self == *other
    }
}

#[derive(Debug, Clone)]
pub struct RepeatedSkillEffect {
    // Could we avoid to clone each time?
    pub skill_id: String,
    pub skill_type: SkillType,
    pub targets_group: SkillTargetsGroup,

    pub max_repeat: u8,
    pub amount_repeat: u8,
    pub elapsed_cooldown: Cooldown,

    pub already_hit: HashSet<CharacterId>, // TODO: Add wave?
}
