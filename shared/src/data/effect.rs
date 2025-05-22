use serde::{Deserialize, Serialize};

// TODO: use strum_macros::EnumIter;
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

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum EffectModifier {
    Flat,
    Multiplier,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum StatType {
    // Local
    LocalAttackDamage,
    LocalMinDamage(DamageType),
    LocalMaxDamage(DamageType),
    LocalCritChances,
    LocalCritDamage,
    LocalAttackSpeed,
    LocalArmor,
    LocalBlock,
    // Global
    GlobalLife,
    GlobalLifeRegen,
    GlobalMana,
    GlobalManaRegen,
    GlobalArmor(DamageType),
    GlobalBlock,
    GlobalSpellDamage,
    GlobalSpellPower,
    GlobalDamage(DamageType),
    GlobalAttackDamage,
    GlobalCritChances,
    GlobalCritDamage,
    GlobalAttackSpeed,
    GlobalSpellSpeed,
    GlobalSpeed,
    GlobalMovementSpeed,
    GlobalGoldFind,
    // TODO: ReducedManaCost?
    // TODO: TriggerSkill (effect trigger + Box Skill) => separate because cannot be hashed/copy etc
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct StatEffect {
    pub stat: StatType,
    pub modifier: EffectModifier,
    pub value: f64,
}
