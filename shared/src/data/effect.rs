use serde::{Deserialize, Serialize};

// TODO: use strum_macros::EnumIter;
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DamageType {
    Physical,
    Fire,
}

impl Default for DamageType {
    fn default() -> Self {
        DamageType::Physical
    }
}

impl DamageType {
    pub fn iter() -> impl Iterator<Item = DamageType> {
        [DamageType::Physical, DamageType::Fire].into_iter()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EffectModifier {
    Flat,
    Multiplier,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EffectStat {
    // Local
    LocalAttackDamage,
    LocalMinDamage(DamageType),
    LocalMaxDamage(DamageType),
    LocalCritChances,
    LocalCritDamage,
    LocalAttackSpeed,
    LocalArmor,
    // Global
    GlobalLife,
    GlobalLifeRegen,
    GlobalMana,
    GlobalManaRegen,
    GlobalArmor,
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
}
