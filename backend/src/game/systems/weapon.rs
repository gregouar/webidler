use shared::data::{
    item::{ItemCategory, ItemSpecs, WeaponSpecs},
    skill::{SkillSpecs, TargetType},
};

pub fn update_weapon_specs(weapon_specs: &mut WeaponSpecs) {
    weapon_specs.cooldown = weapon_specs.base_cooldown;
    weapon_specs.min_damage = weapon_specs.base_min_damage;
    weapon_specs.max_damage = weapon_specs.base_max_damage;

    for prefix in weapon_specs.magic_prefixes.iter() {
        match prefix {
            shared::data::item::WeaponMagicPrefix::AttackSpeed(x) => {
                weapon_specs.cooldown *= 1.0 - x;
            }
            shared::data::item::WeaponMagicPrefix::AttackDamages(x) => {
                weapon_specs.min_damage *= 1.0 + x;
                weapon_specs.max_damage *= 1.0 + x;
            }
        }
    }

    for suffix in weapon_specs.magic_suffixes.iter() {
        match suffix {
            shared::data::item::WeaponMagicSuffix::GoldFind(_) => {}
        }
    }
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
        cooldown: weapon_specs.cooldown,
        mana_cost: 0.0,
        range: weapon_specs.range,
        target_type: TargetType::Enemy,
        shape: weapon_specs.shape,
        min_damages: weapon_specs.min_damage,
        max_damages: weapon_specs.max_damage,
    })
}
