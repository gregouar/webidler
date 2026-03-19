use std::vec;
use strum::IntoEnumIterator;

use shared::data::{
    chance::{Chance, ChanceRange},
    character_status::StatusSpecs,
    item::{ArmorSpecs, ItemBase, ItemModifiers, ItemSpecs, WeaponSpecs},
    item_affix::AffixEffectScope,
    modifier::Modifier,
    skill::{
        ApplyStatusEffect, BaseSkillSpecs, DamageType, SkillEffect, SkillEffectType,
        SkillTargetsGroup, SkillType, TargetType,
    },
    stat_effect::{LuckyRollType, MinMax, StatEffect, StatType},
    values::NonNegative,
};

use crate::game::data::items_store::ItemsStore;

const WEAPON_POISON_DAMAGE_DURATION: f64 = 2.0;

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
                true,
                // &items_store.signature_key,
            )
        })
}

pub fn create_item_specs(
    base: ItemBase,
    modifiers: ItemModifiers,
    old_game: bool,
    // signature_key: &HmacKey,
) -> ItemSpecs {
    let effects: Vec<StatEffect> = (&modifiers.aggregate_effects(AffixEffectScope::Local)).into();

    // TODO: convert local StatType::LifeOnHit(hit_trigger) to item linked trigger

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
        old_game,
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
        match effect.stat {
            StatType::Speed(Some(SkillType::Attack) | None) => {
                weapon_specs.cooldown.apply_negative_effect(effect)
            }
            StatType::Damage {
                skill_type: Some(SkillType::Attack) | None,
                damage_type,
                min_max,
            } => match damage_type {
                Some(damage_type) => {
                    let value = weapon_specs.damage.entry(damage_type).or_default();
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
            },
            StatType::CritChance(Some(SkillType::Attack) | None) => {
                weapon_specs.crit_chance.value.apply_effect(effect)
            }
            StatType::CritDamage(Some(SkillType::Attack) | None) => {
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
                        let value = weapon_specs.damage.entry(damage_type).or_default();
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

    // weapon_specs.damage.retain(|_, value| {
    //     value.max = value.max.evaluate().max(0.0).into();
    //     value.min = value.min.evaluate().max(0.0).into();
    //     value.clamp();

    //     value.max.evaluate() > 0.0
    // });

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
            StatType::Armor(Some(DamageType::Physical)) => armor_specs.armor.apply_effect(effect),
            StatType::Block(Some(SkillType::Attack) | None) => {
                armor_specs.block.apply_effect(effect);
            }
            _ => {}
        }
    }

    armor_specs
}

pub fn make_weapon_skill(item_level: u16, weapon_specs: &WeaponSpecs) -> BaseSkillSpecs {
    let effects = vec![
        SkillEffect {
            effect_type: SkillEffectType::FlatDamage {
                damage: weapon_specs
                    .damage
                    .iter()
                    .filter(|(k, _)| **k != DamageType::Poison)
                    .map(|(&k, &v)| {
                        (
                            k,
                            ChanceRange {
                                min: v.min.as_new_base(),
                                max: v.max.as_new_base(),
                                lucky_chance: v.lucky_chance.as_new_base(),
                            },
                        )
                    })
                    .collect(),
                crit_chance: weapon_specs.crit_chance,
                crit_damage: weapon_specs.crit_damage,
                unblockable: false,
            },
            success_chance: Chance::new_sure(),
            ignore_stat_effects: Default::default(),
            conditional_modifiers: Vec::new(),
        },
        SkillEffect {
            effect_type: SkillEffectType::ApplyStatus {
                duration: ChanceRange {
                    min: NonNegative::new(WEAPON_POISON_DAMAGE_DURATION).into(),
                    max: NonNegative::new(WEAPON_POISON_DAMAGE_DURATION).into(),
                    lucky_chance: Default::default(),
                },
                statuses: vec![ApplyStatusEffect {
                    status_type: StatusSpecs::DamageOverTime {
                        damage_type: DamageType::Poison,
                    },
                    value: weapon_specs
                        .damage
                        .get(&DamageType::Poison)
                        .copied()
                        .unwrap_or_default(),
                    cumulate: false,
                    unavoidable: false,
                    replace_on_value_only: false,
                }],
            },
            success_chance: Chance::new_sure(),
            ignore_stat_effects: Default::default(),
            conditional_modifiers: Vec::new(),
        },
    ];

    // if let Some(&value) = weapon_specs.damage.get(&DamageType::Poison) {
    //     effects.push(SkillEffect {
    //         effect_type: SkillEffectType::ApplyStatus {
    //             duration: ChanceRange {
    //                 min: NonNegative::new(WEAPON_POISON_DAMAGE_DURATION).into(),
    //                 max: NonNegative::new(WEAPON_POISON_DAMAGE_DURATION).into(),
    //                 lucky_chance: Default::default(),
    //             },
    //             statuses: vec![ApplyStatusEffect {
    //                 status_type: StatusSpecs::DamageOverTime {
    //                     damage_type: DamageType::Poison,
    //                 },
    //                 value,
    //                 cumulate: false,
    //                 unavoidable: false,
    //                 replace_on_value_only: false,
    //             }],
    //         },
    //         success_chance: Chance::new_sure(),
    //         ignore_stat_effects: Default::default(),
    //         conditional_modifiers: Vec::new(),
    //     });
    // }

    BaseSkillSpecs {
        skill_id: "weapon_attack".to_string(),
        name: "Weapon Attack".to_string(),
        icon: "skills/attack.svg".to_string(),
        description: "A simple attack with your weapon.".to_string(),
        skill_type: SkillType::Attack,
        cooldown: *weapon_specs.cooldown,
        mana_cost: Default::default(),
        upgrade_cost: 10.0 + 0.5 * item_level as f64,
        upgrade_effects: vec![StatEffect {
            stat: StatType::Damage {
                skill_type: None,
                damage_type: None,
                min_max: None,
            },
            modifier: Modifier::More,
            value: 30.0,
            bypass_ignore: true,
        }],
        modifier_effects: Default::default(),
        targets: vec![SkillTargetsGroup {
            range: weapon_specs.range,
            target_type: TargetType::Enemy,
            shape: weapon_specs.shape,
            target_dead: false,
            repeat: Default::default(),
            effects,
        }],
        triggers: Default::default(),
        auto_use_conditions: Default::default(),
        ignore_stat_effects: Default::default(),
    }
}
