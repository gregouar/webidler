use shared::{
    constants::{MAX_POWER_SHARD_LEVEL_BASE, POWER_SHARD_LEVELS_NEEDED},
    data::{
        area::AreaLevel,
        item::{
            ArmorSpecs, ItemBase, ItemModifiers, ItemRarity, ItemSlot, ItemSpecs, MapSpecs,
            WeaponSpecs,
        },
        item_affix::{AffixEffect, AffixEffectScope, AffixType, ItemAffix, aggregate_effects},
        modifier::Modifier,
        skill::{BaseSkillSpecs, DamageType, SkillType},
        stat_effect::{
            DamageStatType, LuckyRollType, Matchable, MinMax, StatEffect, StatSkillFilter,
            StatType, ToDamageTypes, compare_options,
        },
    },
};

use crate::game::data::{items_store::ItemsStore, master_store::SkillsStore};

pub fn init_item_specs_from_store(
    items_store: &ItemsStore,
    item_modifiers: ItemModifiers,
) -> Option<ItemSpecs> {
    items_store
        .items
        .get(&item_modifiers.base_item_id)
        .map(|item_base| create_item_specs(items_store, item_base, item_modifiers, 0.0))
}

pub fn create_item_specs(
    items_store: &ItemsStore,
    item_base: &ItemBase,
    mut modifiers: ItemModifiers,
    gold_price: f64,
    // signature_key: &HmacKey,
) -> ItemSpecs {
    items_store.items.attach_key(&mut modifiers.base_item_id);
    let affixes = compute_affixes(items_store, item_base, &modifiers);
    let required_level = item_base.min_area_level.max(
        affixes
            .iter()
            .map(|affix| affix.item_level)
            .max()
            .unwrap_or_default(),
    );
    let effects: Vec<StatEffect> =
        (&aggregate_effects(&affixes, AffixEffectScope::Local, false)).into();

    let mut extra_slots = item_base.extra_slots.clone();
    for effect in effects.iter() {
        if let StatType::RemoveSlotUse(item_slot) = &effect.stat {
            extra_slots.remove(item_slot);
        }
    }

    ItemSpecs {
        slot: item_base.slot,
        extra_slots,
        categories: item_base.categories.clone(),
        weapon_specs: item_base
            .weapon_specs
            .clone()
            .map(|weapon_specs| compute_weapon_specs(weapon_specs, modifiers.quality, &effects)),
        armor_specs: item_base
            .armor_specs
            .clone()
            .map(|armor_specs| compute_armor_specs(armor_specs, modifiers.quality, &effects)),
        map_specs: item_base
            .map_specs
            .clone()
            .map(|map_specs| compute_map_specs(map_specs, &modifiers, &affixes)),
        rune_specs: item_base.rune_specs.clone(),
        modifiers,
        affixes,
        required_level,
        gold_price,
    }

    // if let Ok(serialized_item_specs) = rmp_serde::to_vec(&item_specs) {
    //     item_specs.signature = signature::compute_hmac(&serialized_item_specs, signature_key);
    // }

    // item_specs
}

pub fn compute_affixes(
    items_store: &ItemsStore,
    base: &ItemBase,
    modifiers: &ItemModifiers,
) -> Vec<ItemAffix> {
    let mut affixes = modifiers
        .affixes
        .iter()
        .filter_map(|roll| {
            if let Some(source_base_item_id) = &roll.base_item_id {
                let source_base = items_store.items.get(source_base_item_id).or_else(|| {
                    tracing::warn!(
                        source_base_item_id = source_base_item_id.to_string(),
                        affix_id = roll.affix_id,
                        item_base_id = modifiers.base_item_id.to_string(),
                        "missing source base item for item affix"
                    );
                    None
                })?;
                let effect = source_base.affixes.get(&roll.affix_id).or_else(|| {
                    tracing::warn!(
                        source_base_item_id = source_base_item_id.to_string(),
                        affix_id = roll.affix_id,
                        item_base_id = modifiers.base_item_id.to_string(),
                        "missing item affix blueprint on source base item"
                    );
                    None
                })?;
                return Some(ItemAffix {
                    base_item_id: roll.base_item_id.clone(),
                    affix_id: roll.affix_id.clone(),
                    affix_type: roll.affix_type,
                    effects: vec![compute_affix_effect(
                        &effect.effect,
                        roll.value,
                        (roll.affix_type == AffixType::Base).then_some(modifiers.quality),
                    )],
                    item_level: source_base.min_area_level,
                });
            }

            let blueprint = items_store.affixes_table.get(&roll.affix_id).or_else(|| {
                tracing::warn!(
                    affix_id = roll.affix_id,
                    base_item_id = modifiers.base_item_id.to_string(),
                    "missing item affix blueprint"
                );
                None
            })?;
            Some(ItemAffix {
                base_item_id: None,
                affix_id: roll.affix_id.clone(),
                affix_type: roll.affix_type,
                effects: blueprint
                    .effects
                    .iter()
                    .map(|effect| compute_affix_effect(effect, roll.value, None))
                    .collect(),
                item_level: blueprint.item_level,
            })
        })
        .collect::<Vec<_>>();

    affixes.extend(compute_upgrade_affix(base, modifiers));
    affixes
}

fn compute_affix_effect(
    blueprint: &shared::data::item_affix::AffixEffectBlueprint,
    roll: u8,
    quality: Option<f32>,
) -> AffixEffect {
    let value = (blueprint.value.min
        + (blueprint.value.max - blueprint.value.min) * roll.clamp(0, 100) as f64 * 0.01)
        * (1.0 + quality.unwrap_or_default() as f64 * 0.01);

    AffixEffect {
        scope: blueprint.scope,
        stat_effect: StatEffect {
            stat: blueprint.stat.clone(),
            modifier: blueprint.modifier,
            value: value.round(),
            bypass_ignore: blueprint.bypass_ignore,
        },
    }
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
            }) =>
            {
                for damage_type in damage_type.to_damage_types() {
                    if compare_options(is_hit, &Some(!matches!(damage_type, DamageType::Poison))) {
                        let value = weapon_specs.damage.entry(*damage_type).or_default();
                        if let Some(MinMax::Min) | None = min_max {
                            value.min.apply_effect(effect);
                        }
                        if let Some(MinMax::Max) | None = min_max {
                            value.max.apply_effect(effect);
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
            StatType::Armor(Some(DamageStatType::Physical)) => {
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

fn compute_map_specs(
    mut map_specs: MapSpecs,
    modifiers: &ItemModifiers,
    affixes: &[ItemAffix],
) -> MapSpecs {
    map_specs.max_power_shard_level = (modifiers.rarity != ItemRarity::Unique).then(|| {
        let max_power_shard_level = MAX_POWER_SHARD_LEVEL_BASE
            + map_specs.max_power_shard_level.unwrap_or_default()
            + affixes
                .iter()
                .map(|affix| affix.item_level)
                .sum::<AreaLevel>()
                / 4;

        max_power_shard_level - max_power_shard_level % POWER_SHARD_LEVELS_NEEDED
    });
    map_specs
}

fn compute_upgrade_affix(base: &ItemBase, item_modifiers: &ItemModifiers) -> Option<ItemAffix> {
    (item_modifiers.upgrade_level > 0 && !base.upgrade_effects.is_empty()).then(|| ItemAffix {
        base_item_id: None,
        affix_id: "empowered".into(),
        affix_type: AffixType::Upgrade,
        effects: base
            .upgrade_effects
            .iter()
            .cloned()
            .map(|upgrade_effect| AffixEffect {
                scope: upgrade_effect.scope,
                stat_effect: StatEffect {
                    value: upgrade_effect.stat_effect.value * item_modifiers.upgrade_level as f64,
                    ..upgrade_effect.stat_effect
                },
            })
            .collect(),
        item_level: base
            .upgrade_levels
            .get(item_modifiers.upgrade_level.saturating_sub(1) as usize)
            .copied()
            .unwrap_or_default(),
    })
}

pub fn make_weapon_skill(
    skills_store: &SkillsStore,
    item_slot: ItemSlot,
) -> Option<(String, BaseSkillSpecs)> {
    let skill_id = item_slot_to_skill_id(item_slot);
    let base_skill_specs = skills_store.get(skill_id).cloned()?;

    Some((skill_id.to_string(), base_skill_specs))
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
