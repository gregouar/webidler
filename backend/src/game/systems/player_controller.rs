use anyhow::Result;

use serde::{Deserialize, Serialize};
use shared::{
    computations,
    constants::{PLAYER_LIFE_PER_LEVEL, SKILL_BASE_COST, SKILL_COST_FACTOR},
    data::{
        area::{AreaSpecs, AreaState, AreaThreat},
        character::CharacterId,
        item::{ItemSlot, ItemSpecs, WeaponSpecs},
        monster::{MonsterRarity, MonsterSpecs},
        player::{
            PlayerBaseSkill, PlayerBaseSpecs, PlayerInventory, PlayerResources, PlayerSpecs,
            PlayerState,
        },
        skill::BaseSkillSpecs,
    },
};

use crate::{
    game::{
        data::{event::EventsQueue, master_store::SkillsStore},
        systems::{characters_controller, inventory_controller, player_updater, stats_updater},
    },
    rest::AppError,
};

use super::{characters_controller::Target, items_controller, skills_controller};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlayerController {
    pub auto_skills: Vec<bool>,

    #[serde(skip_serializing, skip_deserializing)]
    pub use_skills: Vec<usize>,
}

impl PlayerController {
    pub fn init(specs: &PlayerBaseSpecs) -> Self {
        PlayerController {
            auto_skills: Vec::with_capacity(specs.max_skills as usize),
            use_skills: Vec::with_capacity(specs.max_skills as usize),
        }
    }

    pub fn reset(&mut self) {
        self.use_skills.clear();
    }

    pub fn control_player<'a>(
        &mut self,
        events_queue: &mut EventsQueue,
        area_threat: &AreaThreat,
        player_base_specs: &'a PlayerBaseSpecs,
        player_specs: &'a PlayerSpecs,
        player_state: &'a mut PlayerState,
        monsters: &mut [Target<'a>],
    ) {
        if !player_state.character_state.is_alive || player_state.character_state.is_stunned() {
            return;
        }

        let no_auto_use: Vec<_> = player_base_specs
            .skills
            .values()
            .map(|player_base_skill| {
                player_base_skill
                    .base_skill_specs
                    .auto_use_conditions
                    .iter()
                    .any(|condition| {
                        stats_updater::check_condition(
                            area_threat,
                            &player_specs.character_specs.character_attrs,
                            &player_state.character_state,
                            condition,
                        ) == 0.0
                    })
            })
            .collect();

        let mut mana_available = characters_controller::mana_available(
            &player_specs.character_specs.character_attrs,
            &player_state.character_state,
        );

        let mut player = (
            CharacterId::Player,
            (
                &player_specs.character_specs,
                &mut player_state.character_state,
            ),
        );

        let mut friends = vec![];

        skills_controller::repeat_skills(events_queue, &mut player, &mut friends, monsters);

        let min_mana_needed = if player_specs
            .character_specs
            .character_attrs
            .take_from_mana_before_life
            .get()
            > 0.0
            || player_specs
                .character_specs
                .character_attrs
                .take_from_life_before_mana
                .get()
                > 0.0
        {
            0.0
        } else {
            player_specs
                .character_specs
                .skills_specs
                .iter()
                .take(player_base_specs.max_skills as usize)
                .map(|s| s.mana_cost.get())
                .max_by(|a, b| a.total_cmp(b))
                .unwrap_or_default()
        };

        for (i, (skill_specs, no_auto_use)) in player_specs
            .character_specs
            .skills_specs
            .iter()
            .zip(no_auto_use.into_iter())
            .take(player_base_specs.max_skills as usize)
            .enumerate()
        {
            // Always keep enough mana for a manual trigger, could be optional
            if (!self.auto_skills.get(i).unwrap_or(&false)
                || no_auto_use
                || (skill_specs.mana_cost.get() > 0.0
                    && mana_available.get() < min_mana_needed + skill_specs.mana_cost.get()))
                && !self.use_skills.contains(&i)
            {
                continue;
            }

            mana_available =
                skills_controller::use_skill(events_queue, i, &mut player, &mut friends, monsters);
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
    let gold_reward = (monster_specs.reward_factor * player_specs.gold_find.get() * 0.01).round();
    let gems_reward = if let MonsterRarity::Champion = monster_specs.rarity {
        area_state.last_champion_spawn = area_state.area_level;
        ((area_state.area_level + *area_specs.item_level_modifier + *area_specs.power_level) as f64
            / 5.0
            * *area_specs.gems_find
            * 0.01)
            .floor()
    } else {
        0.0
    };
    player_resources.gold += gold_reward;
    player_resources.gold_total += gold_reward;
    player_resources.gems += gems_reward;
    player_resources.experience += monster_specs.power_factor.round();

    (gold_reward, gems_reward)
}

pub fn level_up(
    player_base_specs: &mut PlayerBaseSpecs,
    player_state: &mut PlayerState,
    player_resources: &mut PlayerResources,
) -> bool {
    if player_resources.experience < player_base_specs.experience_needed
        || player_base_specs.level >= player_base_specs.max_level
    {
        return false;
    }

    player_resources.experience -= player_base_specs.experience_needed;
    level_up_no_cost(player_base_specs, player_state, player_resources);

    true
}

pub fn level_up_no_cost(
    player_base_specs: &mut PlayerBaseSpecs,
    player_state: &mut PlayerState,
    player_resources: &mut PlayerResources,
) {
    player_base_specs.level += 1;
    player_base_specs.experience_needed = computations::player_level_up_cost(player_base_specs);

    player_resources.passive_points += 1;
    player_state.character_state.life += PLAYER_LIFE_PER_LEVEL.into();
    player_base_specs.character_attrs =
        player_updater::base_player_character_attrs(player_base_specs.level);
}

pub fn equip_item_from_bag(
    player_base_specs: &mut PlayerBaseSpecs,
    player_inventory: &mut PlayerInventory,
    player_state: &mut PlayerState,
    player_controller: &mut PlayerController,
    item_index: u8,
) -> Result<(), AppError> {
    let (new_item, old_item) = inventory_controller::equip_item_from_bag(
        player_base_specs.max_area_level,
        player_inventory,
        item_index,
    )?;

    if let Some(old_item) = old_item
        && let Some(slot) = old_item.base.slot
    {
        unequip_weapon(player_base_specs, player_state, player_controller, slot);
    }

    if let Some(ref weapon_specs) = new_item.weapon_specs
        && let Some(slot) = new_item.base.slot
    {
        equip_weapon(
            player_base_specs,
            player_state,
            player_controller,
            slot,
            new_item.modifiers.level,
            weapon_specs,
        );
    }

    Ok(())
}

pub fn unequip_item_to_bag(
    player_base_specs: &mut PlayerBaseSpecs,
    player_inventory: &mut PlayerInventory,
    player_state: &mut PlayerState,
    player_controller: &mut PlayerController,
    item_slot: ItemSlot,
) -> Result<(), AppError> {
    inventory_controller::unequip_item_to_bag(player_inventory, item_slot)?;
    unequip_weapon(
        player_base_specs,
        player_state,
        player_controller,
        item_slot,
    );
    Ok(())
}

pub fn sell_item_from_bag(
    player_inventory: &mut PlayerInventory,
    player_resources: &mut PlayerResources,
    item_index: u8,
) {
    let item_index = item_index as usize;
    if item_index < player_inventory.bag.len() {
        sell_item(player_resources, &player_inventory.bag.remove(item_index));
    }
}

pub fn sell_item(player_resources: &mut PlayerResources, item_specs: &ItemSpecs) {
    player_resources.gold += item_specs.gold_price;
    player_resources.gold_total += item_specs.gold_price;
}

pub fn init_skills_from_inventory(
    player_base_specs: &mut PlayerBaseSpecs,
    player_inventory: &mut PlayerInventory,
    player_state: &mut PlayerState,
    player_controller: &mut PlayerController,
) {
    for (item_slot, equipped_item) in player_inventory.equipped_items() {
        if let Some(weapon_specs) = equipped_item.weapon_specs.as_ref() {
            equip_weapon(
                player_base_specs,
                player_state,
                player_controller,
                item_slot,
                equipped_item.modifiers.level,
                weapon_specs,
            );
        }
    }
}

fn unequip_weapon(
    player_base_specs: &mut PlayerBaseSpecs,
    player_state: &mut PlayerState,
    player_controller: &mut PlayerController,
    item_slot: ItemSlot,
) {
    let to_remove: Vec<_> = player_base_specs
        .skills
        .values()
        .enumerate()
        .filter_map(|(i, player_skill)| (player_skill.item_slot? == item_slot).then_some(i))
        .collect();

    for skill_index in to_remove.into_iter().rev() {
        unequip_base_skill(
            player_base_specs,
            player_state,
            player_controller,
            skill_index,
        );
    }
}

fn equip_weapon(
    player_base_specs: &mut PlayerBaseSpecs,
    player_state: &mut PlayerState,
    player_controller: &mut PlayerController,
    item_slot: ItemSlot,
    item_level: u16,
    weapon_specs: &WeaponSpecs,
) {
    equip_base_skill(
        player_base_specs,
        player_state,
        player_controller,
        item_slot_to_skill_id(item_slot),
        items_controller::make_weapon_skill(item_level, weapon_specs),
        true,
        Some(item_slot),
    );
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

pub fn equip_base_skill(
    player_base_specs: &mut PlayerBaseSpecs,
    player_state: &mut PlayerState,
    player_controller: &mut PlayerController,
    skill_id: &str,
    base_skill_specs: BaseSkillSpecs,
    auto_use: bool,
    item_slot: Option<ItemSlot>,
) {
    let index = if item_slot.is_some() {
        0
    } else {
        player_base_specs.skills.len()
    };

    player_state
        .character_state
        .skills_states
        .insert(index, Default::default());

    player_base_specs.skills.shift_insert(
        index,
        skill_id.to_string(),
        PlayerBaseSkill {
            item_slot,
            upgrade_level: 1,
            next_upgrade_cost: base_skill_specs.upgrade_cost,
            base_skill_specs,
        },
    );

    player_controller.auto_skills.insert(index, auto_use);
}

pub fn unequip_base_skill(
    player_base_specs: &mut PlayerBaseSpecs,
    player_state: &mut PlayerState,
    player_controller: &mut PlayerController,
    skill_index: usize,
) {
    if skill_index < player_base_specs.skills.len() {
        player_base_specs.skills.shift_remove_index(skill_index);
    }

    if skill_index < player_state.character_state.skills_states.len() {
        player_state
            .character_state
            .skills_states
            .remove(skill_index);
    }

    if skill_index < player_controller.auto_skills.len() {
        player_controller.auto_skills.remove(skill_index);
    }
}

pub fn buy_skill(
    skills_store: &SkillsStore,
    player_base_specs: &mut PlayerBaseSpecs,
    player_state: &mut PlayerState,
    player_controller: &mut PlayerController,
    player_resources: &mut PlayerResources,
    skill_id: &str,
) -> bool {
    if player_resources.gold < player_base_specs.buy_skill_cost
        || player_base_specs.skills.len() >= player_base_specs.max_skills as usize
        || player_base_specs.skills.contains_key(skill_id)
    {
        return false;
    }

    if let Some(base_skill_specs) = skills_store.get(skill_id) {
        equip_base_skill(
            player_base_specs,
            player_state,
            player_controller,
            skill_id,
            base_skill_specs.clone(),
            true,
            None,
        );
        player_resources.gold -= player_base_specs.buy_skill_cost;
        player_base_specs.buy_skill_cost = (if player_base_specs.buy_skill_cost > 0.0 {
            player_base_specs.buy_skill_cost * SKILL_COST_FACTOR
        } else {
            SKILL_BASE_COST * SKILL_COST_FACTOR
        })
        .round();
        true
    } else {
        false
    }
}
