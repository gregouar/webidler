use strum::IntoEnumIterator;

use shared::data::{
    item::{ArmorSpecs, ItemBase, ItemModifiers, ItemSlot, ItemSpecs, WeaponSpecs},
    item_affix::{AffixEffect, AffixEffectScope, AffixType, ItemAffix},
    modifier::Modifier,
    skill::{BaseSkillSpecs, DamageType, SkillType},
    stat_effect::{
        ArmorStatType, LuckyRollType, Matchable, MinMax, StatEffect, StatSkillFilter, StatType,
        compare_options,
    },
};

use crate::{
    game::data::{items_store::ItemsStore, master_store::SkillsStore},
    rest::AppError,
};

pub fn init_item_specs_from_store(
    items_store: &ItemsStore,
    item_modifiers: ItemModifiers,
) -> Option<ItemSpecs> {
    items_store
        .content
        .get(&item_modifiers.base_item_id)
        .map(|base| {
            create_item_specs(
                base.clone(),
                item_modifiers,
                0.0, // &items_store.signature_key,
            )
        })
}

pub fn create_item_specs(
    base: ItemBase,
    mut modifiers: ItemModifiers,
    gold_price: f64,
    // signature_key: &HmacKey,
) -> ItemSpecs {
    compute_upgrade_effects(&base, &mut modifiers);

    let effects: Vec<StatEffect> =
        (&modifiers.aggregate_effects(AffixEffectScope::Local, false)).into();

    // TODO: convert local StatType::LifeOnHit(hit_trigger) to item linked trigger
    // TODO: compute triggers with local effects applied to it

    ItemSpecs {
        required_level: base.min_area_level.max(
            modifiers
                .affixes
                .iter()
                .map(|affix| affix.item_level)
                .max()
                .unwrap_or_default(),
        ),
        // .max(1),
        weapon_specs: base.weapon_specs.as_ref().map(|weapon_specs| {
            compute_weapon_specs(weapon_specs.clone(), modifiers.quality, &effects)
        }),
        armor_specs: base.armor_specs.as_ref().map(|armor_specs| {
            compute_armor_specs(armor_specs.clone(), modifiers.quality, &effects)
        }),
        base,
        modifiers,
        old_game: true,
        gold_price,
        // signature: Default::default(),
    }

    // if let Ok(serialized_item_specs) = rmp_serde::to_vec(&item_specs) {
    //     item_specs.signature = signature::compute_hmac(&serialized_item_specs, signature_key);
    // }

    // item_specs
}

fn compute_weapon_specs(
    mut weapon_specs: WeaponSpecs,
    quality: f32,
    effects: &[StatEffect],
) -> WeaponSpecs {
    weapon_specs.damage.values_mut().for_each(|value| {
        value.min.apply_modifier(quality as f64, Modifier::More);
        value.max.apply_modifier(quality as f64, Modifier::More);
    });

    for effect in effects {
        match &effect.stat {
            StatType::Speed(skill_filter)
                if skill_filter.is_match(&StatSkillFilter {
                    skill_type: Some(SkillType::Attack),
                    ..Default::default()
                }) =>
            {
                weapon_specs.cooldown.apply_negative_effect(effect)
            }
            StatType::Damage {
                skill_filter,
                damage_type,
                min_max,
                is_hit,
            } if skill_filter.is_match(&StatSkillFilter {
                skill_type: Some(SkillType::Attack),
                ..Default::default()
            }) && compare_options(is_hit, &Some(true)) =>
            {
                match damage_type {
                    Some(damage_type) => {
                        let value = weapon_specs.damage.entry(*damage_type).or_default();
                        if let Some(MinMax::Min) | None = min_max {
                            value.min.apply_effect(effect);
                        }
                        if let Some(MinMax::Max) | None = min_max {
                            value.max.apply_effect(effect);
                        }
                    }
                    None => {
                        for damage_type in DamageType::iter() {
                            let value = weapon_specs.damage.entry(damage_type).or_default();
                            if let Some(MinMax::Min) | None = min_max {
                                value.min.apply_effect(effect);
                            }
                            if let Some(MinMax::Max) | None = min_max {
                                value.max.apply_effect(effect);
                            }
                        }
                    }
                }
            }
            StatType::CritChance(skill_filter)
                if skill_filter.is_match(&StatSkillFilter {
                    skill_type: Some(SkillType::Attack),
                    ..Default::default()
                }) =>
            {
                weapon_specs.crit_chance.value.apply_effect(effect)
            }
            StatType::CritDamage(skill_filter)
                if skill_filter.is_match(&StatSkillFilter {
                    skill_type: Some(SkillType::Attack),
                    ..Default::default()
                }) =>
            {
                weapon_specs.crit_damage.apply_effect(effect)
            }
            StatType::Lucky {
                roll_type: LuckyRollType::CritChance,
                ..
            } => weapon_specs.crit_chance.lucky_chance.apply_effect(effect),

            StatType::Lucky {
                roll_type: LuckyRollType::Damage { damage_type },
                ..
            } => {
                match damage_type {
                    Some(damage_type) => {
                        let value = weapon_specs.damage.entry(*damage_type).or_default();
                        value.lucky_chance.apply_effect(effect);
                    }
                    None => {
                        for value in weapon_specs.damage.values_mut() {
                            value.lucky_chance.apply_effect(effect);
                        }
                    }
                };
            }
            _ => {}
        }
    }

    weapon_specs
}

fn compute_armor_specs(
    mut armor_specs: ArmorSpecs,
    quality: f32,
    effects: &[StatEffect],
) -> ArmorSpecs {
    armor_specs
        .armor
        .apply_modifier(quality as f64, Modifier::More);
    for effect in effects {
        match effect.stat {
            StatType::Armor(Some(ArmorStatType::Physical)) => {
                armor_specs.armor.apply_effect(effect)
            }
            StatType::Block(Some(SkillType::Attack) | None) => {
                armor_specs.block.apply_effect(effect);
            }
            _ => {}
        }
    }

    armor_specs
}

fn compute_upgrade_effects(base: &ItemBase, item_modifiers: &mut ItemModifiers) {
    if item_modifiers.upgrade_level > 0 {
        item_modifiers
            .affixes
            .retain(|affix| !matches!(affix.affix_type, AffixType::Upgrade));

        item_modifiers
            .affixes
            .extend(base.upgrade_effects.iter().cloned().map(|upgrade_effect| {
                ItemAffix {
                    name: "Empowered".into(),
                    family: "empowered".into(),
                    tags: Default::default(),
                    affix_type: AffixType::Upgrade,
                    tier: item_modifiers.upgrade_level,
                    effects: [AffixEffect {
                        scope: upgrade_effect.scope,
                        stat_effect: StatEffect {
                            value: upgrade_effect.stat_effect.value
                                * item_modifiers.upgrade_level as f64,
                            ..upgrade_effect.stat_effect
                        },
                    }]
                    .into(),
                    item_level: base
                        .upgrade_levels
                        .get(item_modifiers.upgrade_level.saturating_sub(1) as usize)
                        .copied()
                        .unwrap_or_default(),
                }
            }));

        // item_modifiers.affixes.push(ItemAffix {
        //     name: "Empowered".into(),
        //     family: "empowered".into(),
        //     tags: Default::default(),
        //     affix_type: AffixType::Upgrade,
        //     tier: item_modifiers.upgrade_level,
        //     effects: base
        //         .upgrade_effects
        //         .iter()
        //         .cloned()
        //         .map(|upgrade_effect| AffixEffect {
        //             scope: upgrade_effect.scope,
        //             stat_effect: StatEffect {
        //                 value: upgrade_effect.stat_effect.value
        //                     * item_modifiers.upgrade_level as f64,
        //                 ..upgrade_effect.stat_effect
        //             },
        //         })
        //         .collect(),
        //     item_level: base
        //         .upgrade_levels
        //         .get(item_modifiers.upgrade_level.saturating_sub(1) as usize)
        //         .copied()
        //         .unwrap_or_default(),
        // });
    }
}

pub fn upgrade_item(item: &ItemSpecs) -> Result<ItemSpecs, AppError> {
    let available_upgrade_levels = item
        .base
        .upgrade_levels
        .iter()
        .filter(|l| **l <= item.modifiers.level)
        .count();

    if available_upgrade_levels <= item.modifiers.upgrade_level as usize {
        return Err(AppError::UserError("maximum empower level reached.".into()));
    }

    let mut item_modifiers = item.modifiers.clone();
    item_modifiers.upgrade_level = item_modifiers.upgrade_level.saturating_add(1);

    Ok(create_item_specs(item.base.clone(), item_modifiers, 0.0))
}

pub fn make_weapon_skill(
    skills_store: &SkillsStore,
    item_slot: ItemSlot,
    item_level: u16,
) -> Option<(String, BaseSkillSpecs)> {
    let skill_id = item_slot_to_skill_id(item_slot);
    let base_skill_specs = skills_store.get(skill_id).cloned()?;

    Some((
        skill_id.to_string(),
        BaseSkillSpecs {
            upgrade_cost: 10.0 + 0.5 * item_level as f64,
            ..base_skill_specs
        },
    ))
}

fn item_slot_to_skill_id(item_slot: ItemSlot) -> &'static str {
    match item_slot {
        ItemSlot::Accessory => "accessory_skill",
        ItemSlot::Helmet => "helmet_skill",
        ItemSlot::Amulet => "amulet_skill",
        ItemSlot::Weapon => "weapon_skill",
        ItemSlot::Body => "body_skill",
        ItemSlot::Shield => "shield_skill",
        ItemSlot::Gloves => "gloves_skill",
        ItemSlot::Boots => "boots_skill",
        ItemSlot::Ring => "ring_skill",
    }
}
