use shared::data::{
    item::{ArmorSpecs, ItemSlot, ItemSpecs, WeaponSpecs},
    item_affix::{AffixEffect, AffixEffectType, ItemStat},
    skill::{DamageType, SkillEffect, SkillEffectType, SkillSpecs, SkillType, TargetType},
};

// TODO: Where to call that?
pub fn update_item_specs(mut item_specs: ItemSpecs) -> ItemSpecs {
    let mut effects: Vec<AffixEffect> = item_specs.aggregate_effects();

    effects.sort_by_key(|e| match e.effect_type {
        AffixEffectType::Flat => 0,
        AffixEffectType::Multiplier => 1,
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
            ItemStat::AttackSpeed => match effect.effect_type {
                AffixEffectType::Flat => {
                    weapon_specs.cooldown -= effect.value as f32;
                }
                AffixEffectType::Multiplier => {
                    weapon_specs.cooldown *= 1.0 - effect.value as f32;
                }
            },
            ItemStat::AttackDamage => match effect.effect_type {
                AffixEffectType::Flat => {
                    weapon_specs.min_damage += effect.value;
                    weapon_specs.max_damage += effect.value;
                }
                AffixEffectType::Multiplier => {
                    weapon_specs.min_damage *= 1.0 + effect.value;
                    weapon_specs.max_damage *= 1.0 + effect.value;
                }
            },
            ItemStat::MinAttackDamage => match effect.effect_type {
                AffixEffectType::Flat => weapon_specs.min_damage += effect.value,
                AffixEffectType::Multiplier => weapon_specs.min_damage *= 1.0 + effect.value,
            },
            ItemStat::MaxAttackDamage => match effect.effect_type {
                AffixEffectType::Flat => weapon_specs.max_damage += effect.value,
                AffixEffectType::Multiplier => weapon_specs.max_damage *= 1.0 + effect.value,
            },
            ItemStat::Armor | ItemStat::GoldFind => {}
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
            ItemStat::Armor => match effect.effect_type {
                AffixEffectType::Flat => {
                    armor_specs.armor += effect.value;
                }
                AffixEffectType::Multiplier => {
                    armor_specs.armor *= 1.0 + effect.value;
                }
            },
            ItemStat::AttackSpeed
            | ItemStat::AttackDamage
            | ItemStat::MinAttackDamage
            | ItemStat::MaxAttackDamage
            | ItemStat::GoldFind => {}
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
            },
        }],
    }
}
