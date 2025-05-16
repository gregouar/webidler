use shared::data::{
    item::{ArmorSpecs, ItemSlot, ItemSpecs, WeaponSpecs},
    item_affix::{AffixEffect, EffectModifier, EffectStat},
    skill::{SkillEffect, SkillEffectType, SkillSpecs, SkillType, TargetType},
};

use crate::game::data::items_store::{ItemAdjectivesTable, ItemNounsTable};

use super::{loot_generator::generate_name, stats_controller::ApplyStatModifier};

pub fn update_item_specs(
    mut item_specs: ItemSpecs,
    adjectives: &ItemAdjectivesTable,
    nouns: &ItemNounsTable,
) -> ItemSpecs {
    let name = generate_name(&item_specs, adjectives, nouns);
    item_specs.name = name;

    let mut effects: Vec<AffixEffect> = item_specs.aggregate_effects();

    effects.sort_by_key(|e| match e.modifier {
        EffectModifier::Flat => 0,
        EffectModifier::Multiplier => 1,
    });

    if let Some(ref armor_specs) = item_specs.base.armor_specs {
        item_specs.armor_specs = Some(compute_armor_specs(armor_specs.clone(), &effects));
    }

    if let Some(ref weapon_specs) = item_specs.base.weapon_specs {
        item_specs.weapon_specs = Some(compute_weapon_specs(weapon_specs.clone(), &effects));
    }

    item_specs
}

fn compute_weapon_specs(mut weapon_specs: WeaponSpecs, effects: &[AffixEffect]) -> WeaponSpecs {
    for effect in effects {
        match effect.stat {
            EffectStat::LocalAttackSpeed => weapon_specs
                .cooldown
                .apply_modifier(effect.modifier, -effect.value),
            EffectStat::LocalAttackDamage => {
                for (min, max) in weapon_specs.damage.values_mut() {
                    min.apply_effect(effect);
                    max.apply_effect(effect);
                }
            }
            EffectStat::LocalMinDamage(damage_type) => {
                if let Some((min, _)) = weapon_specs.damage.get_mut(&damage_type) {
                    min.apply_effect(effect)
                }
            }
            EffectStat::LocalMaxDamage(damage_type) => {
                if let Some((_, max)) = weapon_specs.damage.get_mut(&damage_type) {
                    max.apply_effect(effect)
                }
            }
            EffectStat::LocalCritChances => weapon_specs.crit_chances.apply_effect(effect),
            EffectStat::LocalCritDamage => weapon_specs.crit_damage.apply_effect(effect),
            _ => {}
        }
    }

    weapon_specs.cooldown = weapon_specs.cooldown.max(0.0);
    weapon_specs.crit_chances = weapon_specs.crit_chances.min(1.0);
    for (min_damage, max_damage) in weapon_specs.damage.values_mut() {
        *max_damage = max_damage.max(0.0);
        *min_damage = min_damage.max(0.0).min(*max_damage);
    }

    weapon_specs
}

fn compute_armor_specs(mut armor_specs: ArmorSpecs, effects: &[AffixEffect]) -> ArmorSpecs {
    for effect in effects {
        match effect.stat {
            EffectStat::LocalArmor => match effect.modifier {
                EffectModifier::Flat => {
                    armor_specs.armor += effect.value;
                }
                EffectModifier::Multiplier => {
                    armor_specs.armor *= 1.0 + effect.value;
                }
            },
            _ => {}
        }
    }
    armor_specs
}

pub fn make_weapon_skill(item_slot: ItemSlot, weapon_specs: &WeaponSpecs) -> SkillSpecs {
    let effects = weapon_specs
        .damage
        .iter()
        .map(|(damage_type, (min, max))| SkillEffect {
            range: weapon_specs.range,
            target_type: TargetType::Enemy,
            shape: weapon_specs.shape,
            effect_type: SkillEffectType::FlatDamage {
                min: *min,
                max: *max,
                damage_type: *damage_type,
                crit_chances: weapon_specs.crit_chances,
                crit_damage: weapon_specs.crit_damage,
            },
        })
        .collect();

    SkillSpecs {
        name: "Weapon Attack".to_string(),
        icon: "skills/attack.svg".to_string(),
        description: "A swing of your weapon".to_string(),
        skill_type: SkillType::Weapon(item_slot),
        base_cooldown: weapon_specs.cooldown,
        cooldown: 0.0,
        mana_cost: 0.0,
        upgrade_level: 1,
        next_upgrade_cost: 10.0,
        base_effects: effects,
        effects: vec![],
    }
}
