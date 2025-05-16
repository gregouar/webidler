use shared::data::{
    item::{ItemSlot, ItemSpecs, WeaponSpecs},
    item_affix::{AffixEffect, EffectModifier, EffectStat},
    monster::{MonsterSpecs, MonsterState},
    player::{PlayerInventory, PlayerResources, PlayerSpecs, PlayerState},
    skill::{SkillState, SkillType},
};

use crate::game::{data::DataInit, utils::increase_factors};

use super::{items_controller, skills_controller, stats_controller::ApplyStatModifier};

#[derive(Debug, Clone)]
pub struct PlayerController {
    pub auto_skills: Vec<bool>,
    pub use_skills: Vec<usize>,
}

impl PlayerController {
    pub fn init(specs: &PlayerSpecs) -> Self {
        PlayerController {
            auto_skills: specs.auto_skills.clone(),
            use_skills: Vec::with_capacity(specs.skills_specs.len()),
        }
    }

    pub fn reset(&mut self) {
        self.use_skills.clear();
    }

    pub fn control_player(
        &mut self,
        player_specs: &PlayerSpecs,
        player_state: &mut PlayerState,
        monsters: &mut Vec<(&MonsterSpecs, &mut MonsterState)>,
    ) {
        if !player_state.character_state.is_alive {
            return;
        }

        let min_mana_needed = player_specs
            .skills_specs
            .iter()
            .map(|s| s.mana_cost)
            .max_by(|a, b| a.total_cmp(&b))
            .unwrap_or_default();

        for (i, (skill_specs, skill_state)) in player_specs
            .skills_specs
            .iter()
            .zip(player_state.skills_states.iter_mut())
            .enumerate()
        {
            if !skill_state.is_ready || skill_specs.mana_cost > player_state.mana {
                continue;
            }

            // Always keep enough mana for a manual trigger, could be optional
            if (!self.auto_skills.get(i).unwrap_or(&false)
                || player_state.mana < min_mana_needed + skill_specs.mana_cost)
                && !self.use_skills.contains(&i)
            {
                continue;
            }

            if skills_controller::use_skill(
                skill_specs,
                skill_state,
                (
                    &player_specs.character_specs,
                    &mut player_state.character_state,
                ),
                vec![],
                monsters
                    .iter_mut()
                    .map(|(specs, state)| (&specs.character_specs, &mut state.character_state))
                    .collect(),
            ) {
                player_state.mana -= skill_specs.mana_cost;
            }
        }
    }
}

pub fn reward_player(
    player_resources: &mut PlayerResources,
    player_specs: &PlayerSpecs,
    monster_specs: &MonsterSpecs,
) {
    player_resources.gold += monster_specs.power_factor * player_specs.gold_find;
    player_resources.experience += monster_specs.power_factor;
}

pub fn level_up(
    player_specs: &mut PlayerSpecs,
    player_state: &mut PlayerState,
    player_resources: &mut PlayerResources,
) -> bool {
    if player_resources.experience < player_specs.experience_needed {
        return false;
    }

    player_specs.level += 1;
    player_resources.passive_points += 1;
    player_resources.experience -= player_specs.experience_needed;
    player_specs.experience_needed =
        10.0 * increase_factors::exponential(player_specs.level as f64);

    player_state.character_state.health += 10.0;
    player_specs.character_specs.max_life += 10.0;

    player_state.just_leveled_up = true;

    true
}

// TODO: InventoryController ?
pub fn equip_item_from_bag(
    player_specs: &mut PlayerSpecs,
    player_inventory: &mut PlayerInventory,
    player_state: &mut PlayerState,
    item_index: u8,
) {
    let item_index = item_index as usize;
    if let Some(item_specs) = player_inventory.bag.get(item_index) {
        if let Some(old_item_specs) = equip_item(
            player_specs,
            player_inventory,
            player_state,
            item_specs.clone(),
        ) {
            player_inventory.bag[item_index] = old_item_specs;
        } else {
            player_inventory.bag.remove(item_index);
        }
    }
}

/// Equip new item and return old equipped item
pub fn equip_item(
    player_specs: &mut PlayerSpecs,
    player_inventory: &mut PlayerInventory,
    player_state: &mut PlayerState,
    item_specs: ItemSpecs,
) -> Option<ItemSpecs> {
    let old_item = player_inventory.equipped.remove(&item_specs.base.slot);

    if let Some(_) = old_item.as_ref().map(|x| x.weapon_specs.as_ref()).flatten() {
        unequip_weapon(player_specs, player_state, item_specs.base.slot);
    }

    if let Some(ref weapon_specs) = item_specs.weapon_specs {
        equip_weapon(
            player_specs,
            player_state,
            item_specs.base.slot,
            weapon_specs,
        );
    }

    player_inventory
        .equipped
        .insert(item_specs.base.slot, item_specs);

    update_player_stats(player_specs, &player_inventory);

    old_item
}

pub fn sell_item(
    player_inventory: &mut PlayerInventory,
    player_resources: &mut PlayerResources,
    item_index: u8,
) {
    let item_index = item_index as usize;
    if item_index < player_inventory.bag.len() {
        let item_specs = player_inventory.bag.remove(item_index);
        player_resources.gold += 10.0 * increase_factors::exponential(item_specs.level as f64);
    }
}

fn unequip_weapon(
    player_specs: &mut PlayerSpecs,
    player_state: &mut PlayerState,
    item_slot: ItemSlot,
) {
    let to_remove: Vec<_> = player_specs
        .skills_specs
        .iter()
        .enumerate()
        .filter_map(|(i, skill_specs)| {
            if let SkillType::Weapon(slot) = skill_specs.skill_type {
                if slot == item_slot {
                    return Some(i);
                }
            }
            None
        })
        .collect();

    for i in to_remove.into_iter().rev() {
        player_specs.skills_specs.remove(i);
        player_state.skills_states.remove(i);
        player_specs.auto_skills.remove(i);
    }
}

fn equip_weapon(
    player_specs: &mut PlayerSpecs,
    player_state: &mut PlayerState,
    item_slot: ItemSlot,
    weapon_specs: &WeaponSpecs,
) {
    let weapon_skill = items_controller::make_weapon_skill(item_slot, &weapon_specs);

    player_specs.auto_skills.insert(0, true);
    player_state
        .skills_states
        .insert(0, SkillState::init(&weapon_skill));
    player_specs.skills_specs.insert(0, weapon_skill);
}

fn update_player_stats(player_specs: &mut PlayerSpecs, player_inventory: &PlayerInventory) {
    player_specs.character_specs.armor = 0.0;
    player_specs.gold_find = 1.0;

    for item in player_inventory.equipped.values() {
        if let Some(armor_specs) = &item.armor_specs {
            player_specs.character_specs.armor += armor_specs.armor;
        }
    }

    let mut effects: Vec<_> = player_inventory
        .equipped
        .values()
        .flat_map(|i| i.aggregate_effects())
        .collect();

    effects.sort_by_key(|e| match e.modifier {
        EffectModifier::Flat => 0,
        EffectModifier::Multiplier => 1,
    });

    compute_player_specs(player_specs, &effects);
}

fn compute_player_specs(player_specs: &mut PlayerSpecs, effects: &Vec<AffixEffect>) {
    for effect in effects.iter() {
        match effect.stat {
            EffectStat::GlobalLife => player_specs
                .character_specs
                .max_life
                .apply_modifier(effect.modifier, effect.value),
            EffectStat::GlobalLifeRegen => player_specs
                .character_specs
                .life_regen
                .apply_modifier(effect.modifier, effect.value),
            EffectStat::GlobalMana => player_specs
                .max_mana
                .apply_modifier(effect.modifier, effect.value),
            EffectStat::GlobalManaRegen => player_specs
                .mana_regen
                .apply_modifier(effect.modifier, effect.value),
            EffectStat::GlobalArmor => player_specs
                .character_specs
                .armor
                .apply_modifier(effect.modifier, effect.value),
            EffectStat::GlobalDamage(damage_type) => todo!(),
            EffectStat::GlobalAttackDamage => todo!(),
            EffectStat::GlobalSpellDamage => todo!(),
            EffectStat::GlobalCritChances => todo!(),
            EffectStat::GlobalCritDamage => todo!(),
            EffectStat::GlobalAttackSpeed => todo!(),
            EffectStat::GlobalSpellSpeed => todo!(),
            EffectStat::GlobalSpeed => todo!(),
            EffectStat::GlobalMovementSpeed => todo!(),
            EffectStat::GlobalGoldFind => match effect.modifier {
                EffectModifier::Flat => todo!(),
                EffectModifier::Multiplier => player_specs.gold_find *= 1.0 + effect.value,
            },
            EffectStat::LocalAttackSpeed
            | EffectStat::LocalAttackDamage
            | EffectStat::LocalMinDamage(_)
            | EffectStat::LocalMaxDamage(_)
            | EffectStat::LocalArmor
            | EffectStat::LocalCritChances
            | EffectStat::LocalCritDamage => {}
        }
    }
}
