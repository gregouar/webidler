use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

use crate::data::{
    chance::{Chance, ChanceRange},
    character::CharacterId,
    conditional_modifier::{Condition, ConditionalModifier},
    item::ItemCategory,
    modifier::ModifiableValue,
    stat_effect::{Matchable, MinMax, StatConverterSource, StatEffect, StatSkillFilter, StatType},
    trigger::TriggerSpecs,
    values::{Cooldown, NonNegative},
};

pub use super::stat_effect::DamageType;
use super::{character_status::StatusSpecs, item::ItemSlot, stat_effect::DamageMap};

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
    pub upgrade_cost: f64,
    #[serde(default)]
    pub upgrade_effects: Vec<StatEffect>,

    #[serde(default)]
    pub modifier_effects: Vec<ModifierEffect>,

    #[serde(default)]
    pub targets: Vec<SkillTargetsGroup>,
    #[serde(default)]
    pub triggers: Vec<TriggerSpecs>,

    #[serde(default)]
    pub auto_use_conditions: Vec<Condition>,

    // #[serde(default)]
    // pub skill_id: String,
    #[serde(default)]
    pub ignore_stat_effects: HashSet<StatType>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SkillSpecs {
    // pub base: BaseSkillSpecs,
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

    #[serde(default, skip_serializing, skip_deserializing)]
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
        slot: Option<ItemSlot>,
        item_stats: ItemStatsSource,
        #[serde(default)]
        category: Option<ItemCategory>,
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
}

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
        statuses: Vec<ApplyStatusEffect>,
        duration: ChanceRange<ModifiableValue<NonNegative>>,
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ApplyStatusEffect {
    pub status_type: StatusSpecs,
    #[serde(default)]
    pub value: ChanceRange<ModifiableValue<NonNegative>>,
    #[serde(default)]
    pub cumulate: bool,
    #[serde(default)]
    pub replace_on_value_only: bool,
    #[serde(default)]
    pub unavoidable: bool,
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
