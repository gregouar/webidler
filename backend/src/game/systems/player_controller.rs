use shared::data::{
    item::{ItemCategory, ItemSpecs},
    monster::{MonsterSpecs, MonsterState},
    player::{PlayerResources, PlayerSpecs, PlayerState},
    skill::{SkillState, SkillType},
};

use crate::game::data::DataInit;

use super::{increase_factors::exponential_factor, skills_controller, weapon::make_weapon_skill};

pub struct PlayerController {
    pub auto_skills: Vec<bool>,
    pub use_skills: Vec<usize>,
}

impl PlayerController {
    pub fn init(specs: &PlayerSpecs) -> Self {
        PlayerController {
            auto_skills: specs.auto_skills.clone(),
            use_skills: Vec::with_capacity(specs.skill_specs.len()),
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

        for (i, (skill_specs, skill_state)) in player_specs
            .skill_specs
            .iter()
            .zip(player_state.skill_states.iter_mut())
            .enumerate()
        {
            if !skill_state.is_ready || skill_specs.mana_cost > player_state.mana {
                continue;
            }

            if !self.auto_skills.get(i).unwrap_or(&false) && !self.use_skills.contains(&i) {
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

pub fn reward_player(player_resources: &mut PlayerResources, monster_specs: &MonsterSpecs) {
    player_resources.gold += monster_specs.power_factor;
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
    player_specs.experience_needed = 10.0 * exponential_factor(player_specs.level as f64);

    player_state.character_state.health += 10.0;
    player_specs.character_specs.max_health += 10.0;

    true
}

// TODO: InventoryController ?
pub fn equip_item(player_specs: &mut PlayerSpecs, player_state: &mut PlayerState, item_index: u8) {
    let item_index = item_index as usize;
    if let Some(item_specs) = player_specs.inventory.bag.get(item_index) {
        if let Some(old_item_specs) = match item_specs.item_category {
            ItemCategory::Trinket => return, // Cannot equip trinket
            ItemCategory::Weapon(_) => {
                equip_weapon(player_specs, Some(player_state), item_specs.clone())
            }
        } {
            player_specs.inventory.bag[item_index] = old_item_specs;
        } else {
            player_specs.inventory.bag.remove(item_index);
        }
    }
}

pub fn equip_weapon(
    player_specs: &mut PlayerSpecs,
    mut player_state: Option<&mut PlayerState>,
    weapon_specs: ItemSpecs,
) -> Option<ItemSpecs> {
    let old_weapon = player_specs.inventory.weapon_specs.take();

    if let Some(SkillType::Weapon) = player_specs.skill_specs.get(0).map(|x| x.skill_type) {
        player_specs.skill_specs.remove(0);
        if let Some(ref mut player_state) = player_state {
            player_state.skill_states.remove(0);
        }
        player_specs.auto_skills.remove(0);
    }

    if let Some(weapon_skill) = make_weapon_skill(&weapon_specs) {
        player_specs.auto_skills.insert(0, true);
        if let Some(ref mut player_state) = player_state {
            player_state
                .skill_states
                .insert(0, SkillState::init(&weapon_skill));
        }
        player_specs.skill_specs.insert(0, weapon_skill);
    }

    player_specs.inventory.weapon_specs = Some(weapon_specs);

    old_weapon
}
