use crate::data::{skill::DamageType, stat_effect::StatSkillEffectType};

pub const MARBLE_COUNT: u8 = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MarbleBag {
    pub picked_marbles: u32, // Bit flag
    pub remaining_marbles: u8,
}

impl MarbleBag {
    pub const fn new() -> Self {
        Self {
            picked_marbles: 0,
            remaining_marbles: MARBLE_COUNT,
        }
    }
}

impl Default for MarbleBag {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum MarbleRollType {
    Damage { damage_type: DamageType },
    Block,
    Evade,
    CritChance,
    SuccessChance { effect_type: StatSkillEffectType },
    // Restore,
    // StatusDuration,
    // StatusValue,
    // TODO: could add others
}
