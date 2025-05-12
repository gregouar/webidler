use shared::data::{
    item::{ArmorSpecs, ItemSlot, ItemSpecs, WeaponSpecs},
    item_affix::{AffixEffect, AffixEffectModifier, ItemStat},
    skill::{DamageType, SkillEffect, SkillEffectType, SkillSpecs, SkillType, TargetType},
};

use crate::game::data::items_store::{ItemAdjectivesTable, ItemNounsTable};

use super::loot_generator::generate_name;

pub fn update_item_specs(
    mut item_specs: ItemSpecs,
    adjectives: &ItemAdjectivesTable,
    nouns: &ItemNounsTable,
) -> ItemSpecs {
    let name = generate_name(&item_specs, adjectives, nouns);
    item_specs.name = name;

    let mut effects: Vec<AffixEffect> = item_specs.aggregate_effects();

    effects.sort_by_key(|e| match e.modifier {
        AffixEffectModifier::Flat => 0,
        AffixEffectModifier::Multiplier => 1,
    });

    if let Some(ref armor_specs) = item_specs.base.armor_specs {
        item_specs.armor_specs = Some(compute_armor_specs(armor_specs.clone(), &effects));
    }

    if let Some(ref weapon_specs) = item_specs.base.weapon_specs {
        item_specs.weapon_specs = Some(compute_weapon_specs(weapon_specs.clone(), &effects));
    }

    item_specs
}

fn compute_weapon_specs(mut weapon_specs: WeaponSpecs, effects: &Vec<AffixEffect>) -> WeaponSpecs {
    for effect in effects {
        match effect.stat {
            ItemStat::LocalAttackSpeed => match effect.modifier {
                AffixEffectModifier::Flat => {
                    weapon_specs.cooldown -= effect.value as f32;
                }
                AffixEffectModifier::Multiplier => {
                    weapon_specs.cooldown *= 1.0 - effect.value as f32;
                }
            },
            ItemStat::LocalAttackDamage => match effect.modifier {
                AffixEffectModifier::Flat => {
                    weapon_specs.min_damage += effect.value;
                    weapon_specs.max_damage += effect.value;
                }
                AffixEffectModifier::Multiplier => {
                    weapon_specs.min_damage *= 1.0 + effect.value;
                    weapon_specs.max_damage *= 1.0 + effect.value;
                }
            },
            ItemStat::LocalMinAttackDamage => match effect.modifier {
                AffixEffectModifier::Flat => weapon_specs.min_damage += effect.value,
                AffixEffectModifier::Multiplier => weapon_specs.min_damage *= 1.0 + effect.value,
            },
            ItemStat::LocalMaxAttackDamage => match effect.modifier {
                AffixEffectModifier::Flat => weapon_specs.max_damage += effect.value,
                AffixEffectModifier::Multiplier => weapon_specs.max_damage *= 1.0 + effect.value,
            },
            ItemStat::LocalArmor | ItemStat::GlobalGoldFind => {}
        }
    }

    weapon_specs.cooldown = weapon_specs.cooldown.max(0.0);
    weapon_specs.max_damage = weapon_specs.max_damage.max(0.0);
    weapon_specs.min_damage = weapon_specs
        .min_damage
        .max(0.0)
        .min(weapon_specs.max_damage);

    weapon_specs
}

fn compute_armor_specs(mut armor_specs: ArmorSpecs, effects: &Vec<AffixEffect>) -> ArmorSpecs {
    for effect in effects {
        match effect.stat {
            ItemStat::LocalArmor => match effect.modifier {
                AffixEffectModifier::Flat => {
                    armor_specs.armor += effect.value;
                }
                AffixEffectModifier::Multiplier => {
                    armor_specs.armor *= 1.0 + effect.value;
                }
            },
            ItemStat::LocalAttackSpeed
            | ItemStat::LocalAttackDamage
            | ItemStat::LocalMinAttackDamage
            | ItemStat::LocalMaxAttackDamage
            | ItemStat::GlobalGoldFind => {}
        }
    }
    armor_specs
}

pub fn make_weapon_skill(item_slot: ItemSlot, weapon_specs: &WeaponSpecs) -> SkillSpecs {
    SkillSpecs {
        name: "Weapon Attack".to_string(),
        icon: "skills/attack.svg".to_string(),
        description: "A swing of your weapon".to_string(),
        skill_type: SkillType::Weapon(item_slot),
        cooldown: weapon_specs.cooldown,
        mana_cost: 0.0,
        upgrade_level: 1,
        next_upgrade_cost: 10.0,
        effects: vec![SkillEffect {
            range: weapon_specs.range,
            target_type: TargetType::Enemy,
            shape: weapon_specs.shape,
            effect_type: SkillEffectType::FlatDamage {
                min: weapon_specs.min_damage,
                max: weapon_specs.max_damage,
                damage_type: DamageType::Physical,
                crit_chances: weapon_specs.crit_chances,
                crit_damage: weapon_specs.crit_damage,
            },
        }],
    }
}
