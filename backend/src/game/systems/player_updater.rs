use std::{iter, time::Duration};

use shared::{
    constants::PLAYER_LIFE_PER_LEVEL,
    data::{
        area::AreaThreat,
        chance::{Chance, ChanceRange},
        character::{CharacterId, CharacterSize},
        character_status::StatusSpecs,
        item::{SkillRange, SkillShape},
        item_affix::AffixEffectScope,
        modifier::Modifier,
        passive::{PassivesTreeSpecs, PassivesTreeState},
        player::{CharacterSpecs, PlayerInventory, PlayerSpecs, PlayerState},
        skill::{DamageType, RestoreType, SkillEffect, SkillEffectType, SkillType},
        stat_effect::{EffectsMap, StatConverterSource, StatConverterSpecs, StatType},
        trigger::{EventTrigger, HitTrigger, TriggerTarget, TriggeredEffect},
    },
};

use crate::game::{
    data::event::EventsQueue,
    systems::{stats_updater, statuses_controller},
};

use super::{characters_updater, passives_controller, skills_updater};

pub fn base_player_character_specs(name: String, portrait: String, level: u8) -> CharacterSpecs {
    CharacterSpecs {
        name,
        portrait,
        size: CharacterSize::Small,
        position_x: 0,
        position_y: 0,
        max_life: (100.0 + PLAYER_LIFE_PER_LEVEL * (level.saturating_sub(1)) as f64).into(),
        life_regen: 10.0.into(),
        max_mana: 100.0.into(),
        mana_regen: 10.0.into(),
        ..Default::default()
    }
}

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
    benedictions_effects: &EffectsMap,
    area_threat: &AreaThreat,
) {
    let effects_map = EffectsMap::combine_all(
        player_inventory
            .equipped_items()
            .map(|(_, i)| i.modifiers.aggregate_effects(AffixEffectScope::Global))
            .chain(iter::once(benedictions_effects.clone()))
            .chain(passives_controller::generate_effects_map_from_passives(
                passives_tree_specs,
                passives_tree_state,
            ))
            .chain(iter::once(
                statuses_controller::generate_effects_map_from_statuses(
                    &player_state.character_state.statuses,
                ),
            ))
            .chain(iter::once(
                stats_updater::compute_conditional_modifiers(
                    &player_specs.character_specs,
                    &player_state.character_state,
                    &player_specs.character_specs.conditional_modifiers,
                )
                .into(),
            )),
    );

    player_specs.character_specs = base_player_character_specs(
        player_specs.character_specs.name.clone(),
        player_specs.character_specs.portrait.clone(),
        player_specs.level,
    );

    player_specs.gold_find = 100.0.into();
    player_specs.threat_gain = 100.0.into();
    player_specs.movement_cooldown = 3.0.into();

    // TODO: Could we figure out a way to keep the block luck somehow?
    let (total_armor, total_block) = player_inventory
        .equipped_items()
        .map(|(_, item)| item)
        .filter_map(|item| item.armor_specs.as_ref())
        .map(|spec| (spec.armor, spec.block))
        .fold((0.0, 0.0), |(a_sum, b_sum), (a, b)| {
            (a_sum + a.evaluate(), b_sum + b.evaluate())
        });

    player_specs
        .character_specs
        .armor
        .entry(DamageType::Physical)
        .or_default()
        .base += total_armor;
    player_specs
        .character_specs
        .block
        .entry(SkillType::Attack)
        .or_default()
        .value
        .base += total_block;

    player_specs.character_specs.triggers = passives_tree_state
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
        .chain(
            player_specs
                .skills_specs
                .iter()
                .flat_map(|skill_specs| skill_specs.triggers.iter()),
        )
        .map(|trigger_specs| trigger_specs.triggered_effect.clone())
        .chain(
            player_inventory
                .equipped_items()
                .flat_map(|(_, item_specs)| item_specs.base.triggers.iter())
                .map(|trigger_specs| trigger_specs.triggered_effect.clone()),
        )
        .collect();

    compute_player_specs(player_specs, player_inventory, area_threat, effects_map);
}

fn compute_player_specs(
    player_specs: &mut PlayerSpecs,
    player_inventory: &PlayerInventory,
    area_threat: &AreaThreat,
    effects_map: EffectsMap,
) {
    let mut effects = stats_updater::stats_map_to_vec(&effects_map, area_threat);

    let (character_specs, converted_effects) =
        characters_updater::update_character_specs(&player_specs.character_specs, &effects);
    player_specs.character_specs = character_specs;
    player_specs.character_specs.effects = EffectsMap::combine_all(
        iter::once(effects_map).chain(iter::once(converted_effects.clone().into())),
    );
    effects.extend(converted_effects);

    for effect in effects.iter() {
        match effect.stat {
            StatType::MovementSpeed => player_specs.movement_cooldown.apply_negative_effect(effect),
            StatType::GoldFind => player_specs.gold_find.apply_effect(effect),
            StatType::ThreatGain => player_specs.threat_gain.apply_effect(effect),
            // TODO: Move the character specs
            StatType::LifeOnHit { skill_type } | StatType::ManaOnHit { skill_type } => {
                if let Modifier::Flat = effect.modifier {
                    player_specs.character_specs.triggers.push(TriggeredEffect {
                        trigger: EventTrigger::OnHit(HitTrigger {
                            skill_type,
                            range: None,
                            is_crit: None,
                            is_blocked: None,
                            is_hurt: Some(true),
                            is_triggered: Some(false),
                            damage_type: None,
                        }),
                        target: TriggerTarget::Source,
                        skill_range: SkillRange::Any,
                        skill_type: skill_type.unwrap_or_default(),
                        skill_shape: SkillShape::Single,
                        modifiers: Vec::new(),
                        effects: vec![SkillEffect {
                            success_chance: Chance::new_sure(),
                            effect_type: SkillEffectType::Restore {
                                restore_type: if let StatType::LifeOnHit { .. } = effect.stat {
                                    RestoreType::Life
                                } else {
                                    RestoreType::Mana
                                },
                                value: ChanceRange {
                                    min: effect.value.into(),
                                    max: effect.value.into(),
                                    lucky_chance: 0.0.into(),
                                },
                                modifier: Modifier::Flat,
                            },
                            ignore_stat_effects: Default::default(),
                            conditional_modifiers: Default::default(),
                        }],
                        owner: Some(CharacterId::Player),
                        inherit_modifiers: false,
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
            | StatType::TakeFromLifeBeforeMana
            | StatType::Block(_)
            | StatType::BlockDamageTaken
            | StatType::Evade(_)
            | StatType::EvadeDamageTaken
            | StatType::DamageResistance { .. }
            | StatType::StatusResistance { .. }
            | StatType::StatConverter(StatConverterSpecs {
                source:
                    StatConverterSource::MaxLife
                    | StatConverterSource::LifeRegen
                    | StatConverterSource::MaxMana
                    | StatConverterSource::ManaRegen
                    | StatConverterSource::Block(_),
                ..
            })
            | StatType::StatConditionalModifier { .. } => {}
            // Delegate to skills
            StatType::ManaCost { .. }
            | StatType::Damage { .. }
            | StatType::Restore { .. }
            | StatType::CritChance(_)
            | StatType::CritDamage(_)
            | StatType::StatusDuration { .. }
            | StatType::StatusPower { .. }
            | StatType::Speed(_)
            | StatType::Lucky { .. }
            | StatType::StatConverter(StatConverterSpecs {
                source: StatConverterSource::CritDamage | StatConverterSource::Damage { .. },
                ..
            })
            | StatType::SuccessChance { .. }
            | StatType::SkillLevel(_)
            | StatType::SkillTargetModifier { .. }
            | StatType::SkillConditionalModifier { .. } => {}
            // Other
            StatType::StatConverter(StatConverterSpecs {
                source: StatConverterSource::ThreatLevel,
                ..
            })
            | StatType::ItemRarity => {}
        }
    }

    for skill_specs in player_specs.skills_specs.iter_mut() {
        skills_updater::update_skill_specs(
            skill_specs,
            effects.iter(),
            Some(player_inventory),
            area_threat,
        );
    }

    // for trigger_effect in player_specs.character_specs.triggers.iter_mut() {
    //     for effect in trigger_effect.effects.iter_mut() {
    //         skills_updater::compute_skill_specs_effect(
    //             trigger_effect.skill_type,
    //             effect,
    //             effects.iter(),
    //         )
    //     }
    // }
}
