use std::{iter, time::Duration};

use shared::data::{
    character::CharacterId,
    character_status::StatusType,
    item_affix::{AffixEffectScope, Modifier, StatType},
    passive::{PassivesTreeSpecs, PassivesTreeState},
    player::{EquippedSlot, PlayerInventory, PlayerSpecs, PlayerState},
    skill::DamageType,
    stat_effect::EffectsMap,
};

use crate::game::data::event::EventsQueue;

use super::{characters_updater, skills_updater, stats_controller::ApplyStatModifier};

pub fn update_player_state(
    events_queue: &mut EventsQueue,
    elapsed_time: Duration,
    player_specs: &PlayerSpecs,
    player_state: &mut PlayerState,
) {
    if !player_state.character_state.is_alive {
        return;
    }

    characters_updater::update_character_state(
        events_queue,
        elapsed_time,
        CharacterId::Player,
        &player_specs.character_specs,
        &mut player_state.character_state,
    );

    if !player_state.character_state.is_stunned() {
        skills_updater::update_skills_states(
            elapsed_time,
            &player_specs.skills_specs,
            &mut player_state.skills_states,
        );
    }
}

pub fn reset_player(player_state: &mut PlayerState) {
    player_state.just_leveled_up = false;
    characters_updater::reset_character(&mut player_state.character_state);
    skills_updater::reset_skills(&mut player_state.skills_states);
}

// I hate the fact player state influences player specs... But I couldn't figure out a way
// to have it working with the dynamic statuses.
pub fn update_player_specs(
    player_specs: &mut PlayerSpecs,
    player_state: &PlayerState,
    player_inventory: &PlayerInventory,
    passives_tree_specs: &PassivesTreeSpecs,
    passive_tree_state: &PassivesTreeState,
) {
    // TODO: Reset player_specs
    player_specs.character_specs.armor = 0.0;
    player_specs.character_specs.fire_armor = 0.0;
    player_specs.character_specs.poison_armor = 0.0;
    player_specs.character_specs.block = 0.0;
    player_specs.character_specs.max_life = 90.0 + 10.0 * player_specs.level as f64;
    player_specs.character_specs.life_regen = 1.0;
    player_specs.character_specs.max_mana = 100.0;
    player_specs.character_specs.mana_regen = 1.0;
    player_specs.gold_find = 1.0;
    player_specs.movement_cooldown = 2.0;
    player_specs.triggers.clear();

    let equipped_items = player_inventory
        .equipped
        .values()
        .filter_map(|slot| match slot {
            EquippedSlot::MainSlot(item) => Some(item),
            _ => None,
        });

    let (total_armor, total_block) = equipped_items
        .clone()
        .filter_map(|item| item.armor_specs.as_ref())
        .map(|spec| (spec.armor, spec.block))
        .fold((0.0, 0.0), |(a_sum, b_sum), (a, b)| (a_sum + a, b_sum + b));

    player_specs.character_specs.armor += total_armor;
    player_specs.character_specs.block += total_block;

    player_specs.effects = EffectsMap::combine_all(
        equipped_items
            .map(|i| i.aggregate_effects(AffixEffectScope::Global))
            .chain(
                passive_tree_state
                    .purchased_nodes
                    .iter()
                    .filter_map(|node_id| {
                        passives_tree_specs
                            .nodes
                            .get(node_id)
                            .map(|node| -> EffectsMap {
                                EffectsMap(
                                    node.effects
                                        .iter()
                                        .map(|effect| {
                                            ((effect.stat, effect.modifier), effect.value)
                                        })
                                        .collect(),
                                )
                            })
                    }),
            )
            .chain(iter::once(EffectsMap(
                player_state
                    .character_state
                    .statuses
                    .iter()
                    .filter_map(|(s, v)| match s {
                        StatusType::StatModifier {
                            stat,
                            modifier,
                            debuff,
                        } => Some((
                            (*stat, *modifier),
                            v.iter()
                                .map(|s| if *debuff { -s.value } else { s.value })
                                .sum(),
                        )),
                        _ => None,
                    })
                    .collect(),
            ))),
    );

    player_specs.triggers = passive_tree_state
        .purchased_nodes
        .iter()
        .filter_map(|node_id| passives_tree_specs.nodes.get(node_id))
        .flat_map(|node| node.triggers.iter().cloned())
        .collect();

    compute_player_specs(player_specs);
}

fn compute_player_specs(player_specs: &mut PlayerSpecs) {
    let mut effects: Vec<_> = (&player_specs.effects).into();

    effects.sort_by_key(|e| match e.modifier {
        Modifier::Flat => 0,
        Modifier::Multiplier => 1,
    });

    for effect in effects.iter() {
        match effect.stat {
            StatType::Life => player_specs.character_specs.max_life.apply_effect(effect),
            StatType::LifeRegen => player_specs.character_specs.life_regen.apply_effect(effect),
            StatType::Mana => player_specs.character_specs.max_mana.apply_effect(effect),
            StatType::ManaRegen => player_specs.character_specs.mana_regen.apply_effect(effect),
            StatType::Armor(armor_type) => match armor_type {
                DamageType::Physical => player_specs.character_specs.armor.apply_effect(effect),
                DamageType::Fire => player_specs.character_specs.fire_armor.apply_effect(effect),
                DamageType::Poison => player_specs
                    .character_specs
                    .poison_armor
                    .apply_effect(effect),
            },
            StatType::TakeFromManaBeforeLife => player_specs
                .character_specs
                .take_from_mana_before_life
                .apply_effect(effect),
            StatType::Block => player_specs.character_specs.block.apply_effect(effect),
            StatType::MovementSpeed => player_specs.movement_cooldown.apply_inverse_effect(effect),
            StatType::GoldFind => player_specs.gold_find.apply_effect(effect),
            // Delegate to skills
            StatType::Damage { .. }
            | StatType::MinDamage { .. }
            | StatType::MaxDamage { .. }
            | StatType::SpellPower
            | StatType::CritChances(_)
            | StatType::CritDamage(_)
            | StatType::Speed(_) => {}
        }
    }

    for skill_specs in player_specs.skills_specs.iter_mut() {
        skills_updater::update_skill_specs(skill_specs, &effects);
    }
}
