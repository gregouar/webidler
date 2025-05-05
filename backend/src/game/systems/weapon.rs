use shared::data::{
    item::{ItemCategory, ItemSpecs, WeaponSpecs},
    item_affix::{AffixEffect, AffixEffectType, ItemStat},
    skill::{DamageType, SkillEffect, SkillEffectType, SkillSpecs, SkillType, TargetType},
};

// TODO: Where to call that?
pub fn update_weapon_specs(weapon_specs: &mut WeaponSpecs, mut effects: Vec<AffixEffect>) {
    weapon_specs.cooldown = weapon_specs.base_cooldown;
    weapon_specs.min_damage = weapon_specs.base_min_damage;
    weapon_specs.max_damage = weapon_specs.base_max_damage;

    effects.sort_by_key(|e| match e.effect_type {
        AffixEffectType::Flat => 0,
        AffixEffectType::Multiplier => 1,
    });

    for effect in &effects {
        match effect.stat {
            ItemStat::AttackSpeed => match effect.effect_type {
                AffixEffectType::Flat => {
                    weapon_specs.cooldown *= 1.0 - effect.value as f32;
                }
                AffixEffectType::Multiplier => {
                    weapon_specs.cooldown -= effect.value as f32;
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
            ItemStat::GoldFind => {}
        }
    }

    weapon_specs.cooldown = weapon_specs.cooldown.max(0.0);
    weapon_specs.max_damage = weapon_specs.max_damage.max(0.0);
    weapon_specs.min_damage = weapon_specs
        .min_damage
        .max(0.0)
        .min(weapon_specs.max_damage);
}

pub fn make_weapon_skill(item_specs: &ItemSpecs) -> Option<SkillSpecs> {
    let weapon_specs = match &item_specs.item_category {
        ItemCategory::Weapon(w) => w,
        _ => return None,
    };

    Some(SkillSpecs {
        name: "Weapon Attack".to_string(),
        icon: "skills/attack.svg".to_string(),
        description: "A swing of your weapon".to_string(),
        skill_type: SkillType::Weapon,
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
    })
}
