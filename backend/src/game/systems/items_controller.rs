use std::vec;

use shared::data::{
    character_status::StatusSpecs,
    item::{ArmorSpecs, ItemBase, ItemModifiers, ItemSpecs, WeaponSpecs},
    item_affix::AffixEffectScope,
    skill::{
        ApplyStatusEffect, BaseSkillSpecs, DamageType, SkillEffect, SkillEffectType,
        SkillTargetsGroup, SkillType, TargetType,
    },
    stat_effect::{ApplyStatModifier, Modifier, StatEffect, StatType},
};

use crate::game::data::items_store::ItemsStore;

const WEAPON_POISON_DAMAGE_DURATION: f64 = 2.0;

pub fn init_item_specs_from_store(
    items_store: &ItemsStore,
    item_modifiers: ItemModifiers,
) -> Option<ItemSpecs> {
    items_store
        .get(&item_modifiers.base_item_id)
        .map(|base| create_item_specs(base.clone(), item_modifiers, true))
}

pub fn create_item_specs(base: ItemBase, modifiers: ItemModifiers, old_game: bool) -> ItemSpecs {
    let mut effects: Vec<StatEffect> =
        (&modifiers.aggregate_effects(AffixEffectScope::Local)).into();

    effects.sort_by_key(|e| match e.modifier {
        Modifier::Flat => 0,
        Modifier::Multiplier => 1,
    });

    // TODO: convert local StatType::LifeOnHit(hit_trigger) to item linked trigger

    ItemSpecs {
        weapon_specs: base
            .weapon_specs
            .as_ref()
            .map(|weapon_specs| compute_weapon_specs(weapon_specs.clone(), &effects)),
        armor_specs: base
            .armor_specs
            .as_ref()
            .map(|armor_specs| compute_armor_specs(armor_specs.clone(), &effects)),
        base,
        modifiers,
        old_game,
    }
}

fn compute_weapon_specs(mut weapon_specs: WeaponSpecs, effects: &[StatEffect]) -> WeaponSpecs {
    for effect in effects {
        match effect.stat {
            StatType::Speed(Some(SkillType::Attack) | None) => {
                weapon_specs.cooldown.apply_negative_effect(effect)
            }
            StatType::Damage {
                skill_type: Some(SkillType::Attack) | None,
                damage_type,
            } => match damage_type {
                Some(damage_type) => {
                    let (min, max) = weapon_specs.damage.entry(damage_type).or_insert((0.0, 0.0));
                    min.apply_effect(effect);
                    max.apply_effect(effect);
                }
                None => {
                    for (min, max) in weapon_specs.damage.values_mut() {
                        min.apply_effect(effect);
                        max.apply_effect(effect);
                    }
                }
            },
            StatType::MinDamage {
                skill_type: Some(SkillType::Attack) | None,
                damage_type,
            } => {
                match damage_type {
                    Some(damage_type) => {
                        let (min, _) = weapon_specs.damage.entry(damage_type).or_insert((0.0, 0.0));
                        min.apply_effect(effect);
                    }
                    None => {
                        for (min, _) in weapon_specs.damage.values_mut() {
                            min.apply_effect(effect);
                        }
                    }
                };
            }
            StatType::MaxDamage {
                skill_type: Some(SkillType::Attack) | None,
                damage_type,
            } => {
                match damage_type {
                    Some(damage_type) => {
                        let (_, max) = weapon_specs.damage.entry(damage_type).or_insert((0.0, 0.0));
                        max.apply_effect(effect);
                    }
                    None => {
                        for (_, max) in weapon_specs.damage.values_mut() {
                            max.apply_effect(effect);
                        }
                    }
                };
            }
            StatType::CritChances(Some(SkillType::Attack) | None) => {
                weapon_specs.crit_chances.apply_effect(effect)
            }
            StatType::CritDamage(Some(SkillType::Attack) | None) => {
                weapon_specs.crit_damage.apply_effect(effect)
            }
            _ => {}
        }
    }

    weapon_specs.cooldown = weapon_specs.cooldown.max(0.0);
    weapon_specs.crit_chances = weapon_specs.crit_chances.clamp(0.0, 100.0);
    weapon_specs.damage.retain(|_, (min, max)| {
        *min = min.clamp(0.0, *max);
        *max > 0.0
    });

    weapon_specs
}

fn compute_armor_specs(mut armor_specs: ArmorSpecs, effects: &[StatEffect]) -> ArmorSpecs {
    for effect in effects {
        match effect.stat {
            StatType::Armor(DamageType::Physical) => armor_specs.armor.apply_effect(effect),
            StatType::Block => {
                armor_specs.block.apply_effect(effect);
            }
            _ => {}
        }
    }
    armor_specs
}

pub fn make_weapon_skill(item_level: u16, weapon_specs: &WeaponSpecs) -> BaseSkillSpecs {
    let mut effects = vec![SkillEffect {
        effect_type: SkillEffectType::FlatDamage {
            damage: weapon_specs
                .damage
                .iter()
                .filter(|(k, _)| **k != DamageType::Poison)
                .map(|(&k, &v)| (k, v))
                .collect(),
            crit_chances: weapon_specs.crit_chances,
            crit_damage: weapon_specs.crit_damage,
        },
        failure_chances: 0.0,
    }];

    if let Some(&(min_value, max_value)) = weapon_specs.damage.get(&DamageType::Poison) {
        effects.push(SkillEffect {
            effect_type: SkillEffectType::ApplyStatus {
                min_duration: WEAPON_POISON_DAMAGE_DURATION,
                max_duration: WEAPON_POISON_DAMAGE_DURATION,
                statuses: vec![ApplyStatusEffect {
                    status_type: StatusSpecs::DamageOverTime {
                        damage_type: DamageType::Poison,
                        ignore_armor: false,
                    },
                    min_value,
                    max_value,
                    cumulate: true,
                }],
            },
            failure_chances: 0.0,
        });
    }

    BaseSkillSpecs {
        name: "Weapon Attack".to_string(),
        icon: "skills/attack.svg".to_string(),
        description: "A simple attack with your weapon.".to_string(),
        skill_type: SkillType::Attack,
        cooldown: weapon_specs.cooldown,
        mana_cost: 0.0,
        upgrade_cost: 10.0 + 2.0 * item_level as f64,
        upgrade_effects: vec![StatEffect {
            stat: StatType::Damage {
                skill_type: None,
                damage_type: None,
            },
            modifier: Modifier::Multiplier,
            value: 0.5,
        }],
        modifier_effects: vec![],
        targets: vec![SkillTargetsGroup {
            range: weapon_specs.range,
            target_type: TargetType::Enemy,
            shape: weapon_specs.shape,
            target_dead: false,
            effects,
        }],
        triggers: vec![],
    }
}
