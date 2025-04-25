use shared::data::{
    item::{ItemCategory, ItemSpecs},
    skill::{SkillSpecs, TargetType},
};

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
        min_damages: weapon_specs.min_damages,
        max_damages: weapon_specs.max_damages,
    })
}
