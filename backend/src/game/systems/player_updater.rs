use std::{iter, time::Duration};

use shared::data::{
    character::CharacterId,
    character_status::StatusSpecs,
    item::SkillRange,
    item_affix::AffixEffectScope,
    passive::{PassivesTreeSpecs, PassivesTreeState},
    player::{PlayerInventory, PlayerSpecs, PlayerState},
    skill::{DamageType, RestoreType, SkillEffect, SkillEffectType, SkillType},
    stat_effect::{ApplyStatModifier, EffectsMap, Modifier, StatType},
    trigger::{EventTrigger, TriggerTarget, TriggeredEffect},
};

use crate::game::{data::event::EventsQueue, systems::statuses_controller};

use super::{characters_updater, passives_controller, skills_updater};

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
    passives_tree_state: &PassivesTreeState,
) {
    // TODO: Reset player_specs
    player_specs.character_specs.armor.clear();
    player_specs.character_specs.block = 0.0;
    player_specs.character_specs.block_spell = 0.0;
    player_specs.character_specs.block_damage = 0.0;
    player_specs.character_specs.max_life = 90.0 + 10.0 * player_specs.level as f64;
    player_specs.character_specs.life_regen = 10.0;
    player_specs.character_specs.max_mana = 100.0;
    player_specs.character_specs.mana_regen = 10.0;
    player_specs.character_specs.damage_resistance.clear();
    player_specs.gold_find = 1.0;
    player_specs.threat_gain = 100.0;
    player_specs.movement_cooldown = 3.0;
    player_specs.triggers.clear();

    let (total_armor, total_block) = player_inventory
        .equipped_items()
        .map(|(_, item)| item)
        .filter_map(|item| item.armor_specs.as_ref())
        .map(|spec| (spec.armor, spec.block))
        .fold((0.0, 0.0), |(a_sum, b_sum), (a, b)| (a_sum + a, b_sum + b));

    (*player_specs
        .character_specs
        .armor
        .entry(DamageType::Physical)
        .or_default()) += total_armor;
    player_specs.character_specs.block += total_block;

    player_specs.effects = EffectsMap::combine_all(
        player_inventory
            .equipped_items()
            .map(|(_, i)| i.modifiers.aggregate_effects(AffixEffectScope::Global))
            .chain(passives_controller::generate_effects_map_from_passives(
                passives_tree_specs,
                passives_tree_state,
            ))
            .chain(iter::once(
                statuses_controller::generate_effects_map_from_statuses(
                    &player_state.character_state.statuses,
                ),
            )),
    );

    player_specs.triggers = passives_tree_state
        .purchased_nodes
        .iter()
        .filter_map(|node_id| passives_tree_specs.nodes.get(node_id))
        .flat_map(|node| node.triggers.iter())
        .chain(
            player_state
                .character_state
                .statuses
                .iter()
                .filter_map(|(status_specs, _)| match status_specs {
                    StatusSpecs::Trigger(trigger_specs) => Some(trigger_specs.as_ref()),
                    _ => None,
                }),
        )
        .map(|trigger_specs| trigger_specs.triggered_effect.clone())
        .chain(
            player_inventory
                .equipped_items()
                .flat_map(|(_, item_specs)| item_specs.base.triggers.iter())
                .map(|trigger_specs| trigger_specs.triggered_effect.clone()),
        )
        .collect();

    compute_player_specs(player_specs, player_inventory);

    // We only add skills trigger after because they were already increased by skill update
    player_specs.triggers.extend(
        player_specs
            .skills_specs
            .iter()
            .flat_map(|skill_specs| skill_specs.triggers.iter())
            .map(|trigger_specs| trigger_specs.triggered_effect.clone()),
    );
}

fn compute_player_specs(player_specs: &mut PlayerSpecs, player_inventory: &PlayerInventory) {
    let effects = characters_updater::stats_map_to_vec(&player_specs.effects);

    player_specs.character_specs =
        characters_updater::update_character_specs(&player_specs.character_specs, &effects);

    for effect in effects.iter() {
        match effect.stat {
            StatType::MovementSpeed => player_specs.movement_cooldown.apply_negative_effect(effect),
            StatType::GoldFind => player_specs.gold_find.apply_effect(effect),
            StatType::ThreatGain => player_specs.threat_gain.apply_effect(effect),
            // TODO: Move the character specs
            StatType::LifeOnHit(hit_trigger) | StatType::ManaOnHit(hit_trigger) => {
                if let Modifier::Flat = effect.modifier {
                    player_specs.triggers.push(TriggeredEffect {
                        trigger: EventTrigger::OnHit(hit_trigger),
                        target: TriggerTarget::Source,
                        skill_range: SkillRange::Any,
                        skill_type: SkillType::Attack,
                        modifiers: Vec::new(),
                        effects: vec![SkillEffect {
                            failure_chances: 0.0,
                            effect_type: SkillEffectType::Restore {
                                restore_type: if let StatType::LifeOnHit(_) = effect.stat {
                                    RestoreType::Life
                                } else {
                                    RestoreType::Mana
                                },
                                min: effect.value,
                                max: effect.value,
                            },
                            ignore_stat_effects: Default::default(),
                        }],
                    });
                }
                // TODO: Find way to do increase?
                // TODO: For multiplier, should iterate and apply to all trigger matching?
            }
            // /!\ No magic _ to be sure we don't forget when adding new Stats
            // Handled by character
            StatType::Life
            | StatType::LifeRegen
            | StatType::Mana
            | StatType::ManaRegen
            | StatType::Armor(_)
            | StatType::TakeFromManaBeforeLife
            | StatType::Block
            | StatType::BlockSpell
            | StatType::BlockDamageTaken
            | StatType::DamageResistance { .. } => {}
            // Delegate to skills
            StatType::Damage { .. }
            | StatType::MinDamage { .. }
            | StatType::MaxDamage { .. }
            | StatType::Restore(_)
            | StatType::SpellPower
            | StatType::CritChances(_)
            | StatType::CritDamage(_)
            | StatType::StatusDuration { .. }
            | StatType::StatusPower { .. }
            | StatType::Speed(_) => {}
        }
    }

    for skill_specs in player_specs.skills_specs.iter_mut() {
        skills_updater::update_skill_specs(skill_specs, effects.iter(), Some(player_inventory));
    }

    for trigger_effect in player_specs.triggers.iter_mut() {
        for effect in trigger_effect.effects.iter_mut() {
            skills_updater::compute_skill_specs_effect(
                trigger_effect.skill_type,
                effect,
                effects.iter(),
            )
        }
    }
}
