use anyhow::Result;

use shared::{
    computations,
    constants::{
        MONSTER_INCREASE_FACTOR, PLAYER_LIFE_PER_LEVEL, SKILL_BASE_COST, SKILL_COST_FACTOR,
    },
    data::{
        area::{AreaSpecs, AreaState},
        character::CharacterId,
        item::{ItemCategory, ItemRarity, ItemSlot, ItemSpecs, WeaponSpecs},
        monster::{MonsterRarity, MonsterSpecs},
        player::{PlayerInventory, PlayerResources, PlayerSpecs, PlayerState},
        skill::{BaseSkillSpecs, SkillSpecs, SkillState},
    },
};

use crate::{
    game::{
        data::{event::EventsQueue, master_store::SkillsStore, DataInit},
        systems::inventory_controller,
    },
    rest::AppError,
};

use super::{characters_controller::Target, items_controller, skills_controller};

#[derive(Debug, Clone)]
pub struct PlayerController {
    // pub auto_skills: Vec<bool>,
    pub use_skills: Vec<usize>,
    pub preferred_loot: Option<ItemCategory>,
}

impl PlayerController {
    pub fn init(specs: &PlayerSpecs) -> Self {
        PlayerController {
            // auto_skills: specs.auto_skills.clone(),
            use_skills: Vec::with_capacity(specs.skills_specs.len()),
            preferred_loot: None,
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

        let min_mana_needed = if player_specs.character_specs.take_from_mana_before_life > 0.0 {
            0.0
        } else {
            player_specs
                .skills_specs
                .iter()
                .take(player_specs.max_skills as usize)
                .map(|s| s.mana_cost)
                .max_by(|a, b| a.total_cmp(b))
                .unwrap_or_default()
        };

        for (i, (skill_specs, skill_state)) in player_specs
            .skills_specs
            .iter()
            .zip(player_state.skills_states.iter_mut())
            .take(player_specs.max_skills as usize)
            .enumerate()
        {
            // Always keep enough mana for a manual trigger, could be optional
            if (!player_specs.auto_skills.get(i).unwrap_or(&false)
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
    area_specs: &AreaSpecs,
    area_state: &mut AreaState,
) -> (f64, f64) {
    let gold_reward = (monster_specs.reward_factor * player_specs.gold_find * 0.01).round();
    let gems_reward = if let MonsterRarity::Champion = monster_specs.rarity {
        area_state.last_champion_spawn = area_state.area_level;
        ((area_state.area_level + area_specs.item_level_modifier) as f64 / 5.0).floor()
    } else {
        0.0
    };
    player_resources.gold += gold_reward;
    player_resources.gold_total += gold_reward;
    player_resources.gems += gems_reward;
    player_resources.experience += monster_specs.reward_factor.round();

    (gold_reward, gems_reward)
}

pub fn level_up(
    player_specs: &mut PlayerSpecs,
    player_state: &mut PlayerState,
    player_resources: &mut PlayerResources,
) -> bool {
    if player_resources.experience < player_specs.experience_needed {
        return false;
    }

    player_resources.experience -= player_specs.experience_needed;
    level_up_no_cost(player_specs, player_state, player_resources);

    true
}

pub fn level_up_no_cost(
    player_specs: &mut PlayerSpecs,
    player_state: &mut PlayerState,
    player_resources: &mut PlayerResources,
) {
    player_specs.level += 1;
    player_resources.passive_points += 1;
    player_specs.experience_needed = computations::player_level_up_cost(player_specs);

    player_state.character_state.life += PLAYER_LIFE_PER_LEVEL;
}

pub fn equip_item_from_bag(
    player_specs: &mut PlayerSpecs,
    player_inventory: &mut PlayerInventory,
    player_state: &mut PlayerState,
    item_index: u8,
) -> Result<(), AppError> {
    let (new_item, old_item) = inventory_controller::equip_item_from_bag(
        player_specs.max_area_level,
        player_inventory,
        item_index,
    )?;

    if let Some(old_item) = old_item {
        unequip_weapon(player_specs, player_state, old_item.base.slot);
    }

    if let Some(ref weapon_specs) = new_item.weapon_specs {
        equip_weapon(
            player_specs,
            player_state,
            new_item.base.slot,
            new_item.modifiers.level,
            weapon_specs,
        );
    }

    Ok(())
}

pub fn unequip_item_to_bag(
    player_specs: &mut PlayerSpecs,
    player_inventory: &mut PlayerInventory,
    player_state: &mut PlayerState,
    item_slot: ItemSlot,
) -> Result<(), AppError> {
    inventory_controller::unequip_item_to_bag(player_inventory, item_slot)?;
    unequip_weapon(player_specs, player_state, item_slot);
    Ok(())
}

pub fn sell_item_from_bag(
    area_specs: &AreaSpecs,
    player_specs: &PlayerSpecs,
    player_inventory: &mut PlayerInventory,
    player_resources: &mut PlayerResources,
    item_index: u8,
) {
    let item_index = item_index as usize;
    if item_index < player_inventory.bag.len() {
        sell_item(
            area_specs,
            player_specs,
            player_resources,
            &player_inventory.bag.remove(item_index),
        );
    }
}

pub fn sell_item(
    area_specs: &AreaSpecs,
    player_specs: &PlayerSpecs,
    player_resources: &mut PlayerResources,
    item_specs: &ItemSpecs,
) {
    if item_specs.old_game {
        return;
    }

    let gold_reward =
        10.0 * match item_specs.modifiers.rarity {
            ItemRarity::Normal => 1.0,
            ItemRarity::Magic => 2.0,
            ItemRarity::Rare => 4.0,
            ItemRarity::Unique => 8.0,
            ItemRarity::Masterwork => 8.0,
        } * player_specs.gold_find
            * 0.01
            * computations::exponential(
                item_specs
                    .modifiers
                    .level
                    .saturating_sub(area_specs.starting_level + area_specs.item_level_modifier - 1),
                MONSTER_INCREASE_FACTOR,
            );

    player_resources.gold += gold_reward;
    player_resources.gold_total += gold_reward;
}

pub fn init_skills_from_inventory(
    player_specs: &mut PlayerSpecs,
    player_inventory: &mut PlayerInventory,
    player_state: &mut PlayerState,
) {
    for (item_slot, equipped_item) in player_inventory.equipped_items() {
        if let Some(weapon_specs) = equipped_item.weapon_specs.as_ref() {
            equip_weapon(
                player_specs,
                player_state,
                item_slot,
                equipped_item.modifiers.level,
                weapon_specs,
            );
        }
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

    let index = if item_slot.is_some() {
        0
    } else {
        player_specs.skills_specs.len()
    };

    player_state
        .skills_states
        .insert(index, SkillState::init(&skill_specs));
    player_specs.skills_specs.insert(index, skill_specs);
    player_specs.auto_skills.insert(index, auto_use);
}

pub fn buy_skill(
    skills_store: &SkillsStore,
    player_specs: &mut PlayerSpecs,
    player_state: &mut PlayerState,
    player_resources: &mut PlayerResources,
    skill_id: &str,
) -> bool {
    if player_resources.gold < player_specs.buy_skill_cost
        || player_specs.skills_specs.len() >= player_specs.max_skills as usize
        || player_specs.bought_skills.contains(skill_id)
    {
        return false;
    }

    if let Some(base_skill_specs) = skills_store.get(skill_id) {
        equip_skill(
            player_specs,
            player_state,
            base_skill_specs.clone(),
            true,
            None,
        );
        player_resources.gold -= player_specs.buy_skill_cost;
        player_specs.buy_skill_cost = (if player_specs.buy_skill_cost > 0.0 {
            player_specs.buy_skill_cost * SKILL_COST_FACTOR
        } else {
            SKILL_BASE_COST * SKILL_COST_FACTOR
        })
        .round();
        player_specs.bought_skills.insert(skill_id.to_string());
        true
    } else {
        false
    }
}
