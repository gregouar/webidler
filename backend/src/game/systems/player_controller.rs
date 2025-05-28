use anyhow::{anyhow, Result};

use shared::data::{
    character::CharacterId,
    item::{ItemSlot, ItemSpecs, WeaponSpecs},
    monster::MonsterSpecs,
    player::{EquippedSlot, PlayerInventory, PlayerResources, PlayerSpecs, PlayerState},
    skill::{BaseSkillSpecs, SkillSpecs, SkillState},
    world::AreaLevel,
};

use crate::game::{
    data::{event::EventsQueue, DataInit},
    utils::increase_factors,
};

use super::{characters_controller::Target, items_controller, skills_controller};

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

    pub fn control_player<'a>(
        &mut self,
        events_queue: &mut EventsQueue,
        player_specs: &'a PlayerSpecs,
        player_state: &'a mut PlayerState,
        monsters: &mut [Target<'a>],
    ) {
        if !player_state.character_state.is_alive || player_state.character_state.is_stunned() {
            return;
        }

        let mut mana_available = player_state.character_state.mana;

        let mut player = (
            CharacterId::Player,
            (
                &player_specs.character_specs,
                &mut player_state.character_state,
            ),
        );

        let mut friends = vec![];

        let min_mana_needed = player_specs
            .skills_specs
            .iter()
            .map(|s| s.mana_cost)
            .max_by(|a, b| a.total_cmp(b))
            .unwrap_or_default();

        for (i, (skill_specs, skill_state)) in player_specs
            .skills_specs
            .iter()
            .zip(player_state.skills_states.iter_mut())
            .enumerate()
        {
            // Always keep enough mana for a manual trigger, could be optional
            if (!self.auto_skills.get(i).unwrap_or(&false)
                || (skill_specs.mana_cost > 0.0
                    && mana_available < min_mana_needed + skill_specs.mana_cost))
                && !self.use_skills.contains(&i)
            {
                continue;
            }

            mana_available = skills_controller::use_skill(
                events_queue,
                skill_specs,
                skill_state,
                &mut player,
                &mut friends,
                monsters,
            );
        }

        self.reset();
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
    player_specs.experience_needed = 20.0
        * increase_factors::exponential(
            player_specs.level as AreaLevel,
            increase_factors::XP_INCREASE_FACTOR,
        );

    player_state.character_state.life += 10.0;

    player_state.just_leveled_up = true;

    true
}

pub fn equip_item_from_bag(
    player_specs: &mut PlayerSpecs,
    player_inventory: &mut PlayerInventory,
    player_state: &mut PlayerState,
    item_index: u8,
) -> bool {
    let item_index = item_index as usize;
    if let Some(item_specs) = player_inventory.bag.get(item_index) {
        let old_item = match equip_item(
            player_specs,
            player_inventory,
            player_state,
            item_specs.clone(),
        ) {
            Ok(x) => x,
            Err(_) => return false,
        };

        if let Some(old_item_specs) = old_item {
            player_inventory.bag[item_index] = old_item_specs;
        } else {
            player_inventory.bag.remove(item_index);
        }
    }
    true
}

pub fn unequip_item_to_bag(
    player_specs: &mut PlayerSpecs,
    player_inventory: &mut PlayerInventory,
    player_state: &mut PlayerState,
    item_slot: ItemSlot,
) -> bool {
    if player_inventory.bag.len() >= player_inventory.max_bag_size as usize {
        return false;
    }

    if let Some(item) = unequip_item(player_specs, player_inventory, player_state, item_slot) {
        player_inventory.bag.push(item);
    }

    true
}

/// Equip new item and return old equipped item
pub fn equip_item(
    player_specs: &mut PlayerSpecs,
    player_inventory: &mut PlayerInventory,
    player_state: &mut PlayerState,
    item_specs: ItemSpecs,
) -> Result<Option<ItemSpecs>> {
    if item_specs
        .base
        .extra_slots
        .iter()
        .any(|x| match player_inventory.equipped.get(x) {
            Some(EquippedSlot::MainSlot(_)) => true,
            Some(EquippedSlot::ExtraSlot(main_slot)) => *main_slot != item_specs.base.slot,
            None => false,
        })
    {
        return Err(anyhow!("slot unavailable"));
    }

    let old_item = unequip_item(
        player_specs,
        player_inventory,
        player_state,
        item_specs.base.slot,
    );

    if let Some(ref weapon_specs) = item_specs.weapon_specs {
        equip_weapon(
            player_specs,
            player_state,
            item_specs.base.slot,
            item_specs.level,
            weapon_specs,
        );
    }

    for item_slot in item_specs.base.extra_slots.iter() {
        player_inventory
            .equipped
            .insert(*item_slot, EquippedSlot::ExtraSlot(item_specs.base.slot));
    }

    player_inventory.equipped.insert(
        item_specs.base.slot,
        EquippedSlot::MainSlot(Box::new(item_specs)),
    );

    Ok(old_item)
}

pub fn unequip_item(
    player_specs: &mut PlayerSpecs,
    player_inventory: &mut PlayerInventory,
    player_state: &mut PlayerState,
    item_slot: ItemSlot,
) -> Option<ItemSpecs> {
    match player_inventory.equipped.remove(&item_slot) {
        Some(EquippedSlot::MainSlot(old_item)) => {
            for item_slot in old_item.base.extra_slots.iter() {
                player_inventory.equipped.remove(item_slot);
            }
            if old_item.weapon_specs.is_some() {
                unequip_weapon(player_specs, player_state, item_slot);
            }
            Some(*old_item)
        }
        Some(EquippedSlot::ExtraSlot(item_slot)) => {
            unequip_item(player_specs, player_inventory, player_state, item_slot)
        }
        None => None,
    }
}

pub fn sell_item(
    player_specs: &PlayerSpecs,
    player_inventory: &mut PlayerInventory,
    player_resources: &mut PlayerResources,
    item_index: u8,
) {
    let item_index = item_index as usize;
    if item_index < player_inventory.bag.len() {
        let item_specs = player_inventory.bag.remove(item_index);
        player_resources.gold +=
            10.0 * match item_specs.rarity {
                shared::data::item::ItemRarity::Normal => 1.0,
                shared::data::item::ItemRarity::Magic => 2.0,
                shared::data::item::ItemRarity::Rare => 4.0,
                shared::data::item::ItemRarity::Unique => 8.0,
            } * player_specs.gold_find
                * increase_factors::exponential(
                    item_specs.level,
                    increase_factors::MONSTER_INCREASE_FACTOR,
                );
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
        .filter_map(|(i, skill_specs)| (skill_specs.item_slot? == item_slot).then_some(i))
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
    item_level: u16,
    weapon_specs: &WeaponSpecs,
) {
    equip_skill(
        player_specs,
        player_state,
        items_controller::make_weapon_skill(item_level, weapon_specs),
        true,
        Some(item_slot),
    );
}

pub fn equip_skill(
    player_specs: &mut PlayerSpecs,
    player_state: &mut PlayerState,
    base_skill_specs: BaseSkillSpecs,
    auto_use: bool,
    item_slot: Option<ItemSlot>,
) {
    let mut skill_specs = SkillSpecs::init(base_skill_specs);
    skill_specs.item_slot = item_slot;

    player_state
        .skills_states
        .insert(0, SkillState::init(&skill_specs));
    player_specs.skills_specs.insert(0, skill_specs);
    player_specs.auto_skills.insert(0, auto_use);
}
