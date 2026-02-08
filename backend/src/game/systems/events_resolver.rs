use std::iter;

use itertools::all;
use shared::{
    constants::WAVES_PER_AREA_LEVEL,
    data::{
        area::{AreaLevel, ThreatLevel},
        character::CharacterId,
        skill::TargetType,
        trigger::EventTrigger,
    },
};

use crate::game::{
    data::{
        event::{EventsQueue, GameEvent, HitEvent, StatusEvent},
        master_store::MasterStore,
    },
    game_data::GameInstanceData,
    systems::{stats_updater::check_condition, triggers_controller},
};

use super::{
    loot_controller, loot_generator, player_controller, triggers_controller::TriggerContext,
};

pub async fn resolve_events(
    events_queue: &mut EventsQueue,
    game_data: &mut GameInstanceData,
    master_store: &MasterStore,
) {
    let mut trigger_contexts = Vec::new();

    let events = events_queue.consume_events();
    for event in events.iter() {
        match event {
            GameEvent::Hit(hit_event) => {
                handle_hit_event(&mut trigger_contexts, game_data, hit_event)
            }
            GameEvent::Kill { target } => {
                handle_kill_event(&mut trigger_contexts, game_data, *target)
            }
            GameEvent::AreaCompleted {
                area_level,
                is_boss,
            } => handle_area_completed_event(game_data, master_store, *area_level, *is_boss),
            GameEvent::WaveCompleted(area_level) => handle_wave_completed_event(
                &mut trigger_contexts,
                events_queue,
                game_data,
                *area_level,
            ),
            GameEvent::ThreatIncreased(threat_level) => {
                handle_threat_increased_event(&mut trigger_contexts, game_data, *threat_level)
            }
            GameEvent::StatusApplied(status_event) => {
                handle_status_event(&mut trigger_contexts, game_data, status_event)
            }
        }
    }

    triggers_controller::apply_trigger_effects(events_queue, game_data, trigger_contexts);
}

fn handle_hit_event<'a>(
    trigger_contexts: &mut Vec<TriggerContext<'a>>,
    game_data: &mut GameInstanceData,
    hit_event: &'a HitEvent,
) {
    let characters = iter::once((
        CharacterId::Player,
        &game_data.player_specs.read().character_specs,
    ))
    .chain(
        game_data
            .monster_specs
            .iter()
            .enumerate()
            .map(|(idx, monster_specs)| {
                (CharacterId::Monster(idx), &monster_specs.character_specs)
            }),
    );

    for (character_id, character_specs) in characters {
        for triggered_effects in character_specs.triggers.iter() {
            match triggered_effects.trigger {
                EventTrigger::OnHit(_) if hit_event.source == character_id => {}
                EventTrigger::OnTakeHit(_) if hit_event.target == character_id => {}
                _ => continue,
            };

            let hit_trigger = match triggered_effects.trigger {
                EventTrigger::OnHit(ht) | EventTrigger::OnTakeHit(ht) => ht,
                _ => continue,
            };

            if hit_trigger.skill_type.unwrap_or(hit_event.skill_type) == hit_event.skill_type
                && hit_trigger.range.unwrap_or(hit_event.range) == hit_event.range
                && hit_trigger.is_crit.unwrap_or(hit_event.is_crit) == hit_event.is_crit
                && hit_trigger.is_blocked.unwrap_or(hit_event.is_blocked) == hit_event.is_blocked
                && hit_trigger.is_hurt.unwrap_or(hit_event.is_hurt) == hit_event.is_hurt
                && hit_trigger.is_triggered.unwrap_or(hit_event.is_triggered)
                    == hit_event.is_triggered
                && hit_trigger
                    .damage_type
                    .map(|damage_type| {
                        hit_event
                            .damage
                            .get(&damage_type)
                            .copied()
                            .unwrap_or_default()
                            > 0.0
                    })
                    .unwrap_or(true)
            {
                trigger_contexts.push(TriggerContext {
                    trigger: triggered_effects.clone(),
                    source: hit_event.source,
                    target: hit_event.target,
                    hit_context: Some(hit_event),
                    status_context: None,
                    level: game_data.area_state.read().area_level as usize,
                });
            }
        }
    }
}

fn handle_status_event<'a>(
    trigger_contexts: &mut Vec<TriggerContext<'a>>,
    game_data: &mut GameInstanceData,
    status_event: &'a StatusEvent,
) {
    let characters = iter::once((
        CharacterId::Player,
        &game_data.player_specs.read().character_specs,
    ))
    .chain(
        game_data
            .monster_specs
            .iter()
            .enumerate()
            .map(|(idx, monster_specs)| {
                (CharacterId::Monster(idx), &monster_specs.character_specs)
            }),
    );

    for (character_id, character_specs) in characters {
        for triggered_effects in character_specs.triggers.iter() {
            match triggered_effects.trigger {
                EventTrigger::OnApplyStatus(_) if status_event.source == character_id => {}
                // EventTrigger::OnTakeHit(_) if hit_event.target == character_id => {}
                _ => continue,
            };

            let status_trigger = match &triggered_effects.trigger {
                EventTrigger::OnApplyStatus(trigger)
                // | EventTrigger::OnTakeHit(ht) 
                => trigger,
                _ => continue,
            };

            if status_trigger.skill_type.unwrap_or(status_event.skill_type)
                == status_event.skill_type
                && status_trigger
                    .is_triggered
                    .unwrap_or(status_event.is_triggered)
                    == status_event.is_triggered
                && status_trigger
                    .status_type
                    .as_ref()
                    .map(|status_type| status_event.status_type.is_match(status_type))
                    .unwrap_or(true)
            {
                trigger_contexts.push(TriggerContext {
                    trigger: triggered_effects.clone(),
                    source: status_event.source,
                    target: status_event.target,
                    hit_context: None,
                    status_context: Some(status_event),
                    level: game_data.area_state.read().area_level as usize,
                });
            }
        }
    }
}

fn handle_kill_event(
    trigger_contexts: &mut Vec<TriggerContext>,
    game_data: &mut GameInstanceData,
    target: CharacterId,
) {
    match target {
        CharacterId::Monster(monster_index) => {
            game_data.game_stats.monsters_killed += 1;

            if let Some(monster_specs) = game_data.monster_specs.get(monster_index) {
                let (gold_reward, gems_reward) = player_controller::reward_player(
                    game_data.player_resources.mutate(),
                    game_data.player_specs.read(),
                    monster_specs,
                    &game_data.area_blueprint.specs,
                    game_data.area_state.mutate(),
                );
                if let Some(monster_state) = game_data.monster_states.get_mut(monster_index) {
                    monster_state.gold_reward = gold_reward;
                    monster_state.gems_reward = gems_reward;

                    for triggered_effects in game_data
                        .player_specs
                        .read()
                        .character_specs
                        .triggers
                        .iter()
                    {
                        if let EventTrigger::OnKill(kill_trigger) = &triggered_effects.trigger {
                            if all(kill_trigger.conditions.iter(), |condition| {
                                check_condition(
                                    &monster_specs.character_specs,
                                    &monster_state.character_state,
                                    condition,
                                )
                            }) {
                                trigger_contexts.push(TriggerContext {
                                    trigger: triggered_effects.clone(),
                                    source: CharacterId::Player,
                                    target,
                                    hit_context: None,
                                    status_context: None,
                                    level: game_data.area_state.read().area_level as usize,
                                });
                            }
                        }
                    }

                    for (idx, (monster_specs, monster_state)) in game_data
                        .monster_specs
                        .iter()
                        .zip(game_data.monster_states.iter())
                        .enumerate()
                    {
                        let event_target_type = match target {
                            CharacterId::Player => TargetType::Enemy,
                            CharacterId::Monster(event_target_idx) => {
                                if event_target_idx == idx {
                                    TargetType::Me
                                } else {
                                    TargetType::Friend
                                }
                            }
                        };
                        for triggered_effects in &monster_specs.character_specs.triggers {
                            if let EventTrigger::OnDeath(target_type) = triggered_effects.trigger {
                                if target_type == event_target_type
                                    && (monster_state.character_state.is_alive
                                        || target_type == TargetType::Me)
                                {
                                    trigger_contexts.push(TriggerContext {
                                        trigger: triggered_effects.clone(),
                                        source: CharacterId::Player,
                                        target,
                                        hit_context: None,
                                        status_context: None,
                                        level: game_data.area_state.read().area_level as usize,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
        CharacterId::Player => {
            game_data.game_stats.player_deaths += 1;
        }
    }
}

fn handle_area_completed_event(
    game_data: &mut GameInstanceData,
    master_store: &MasterStore,
    area_level: AreaLevel,
    is_boss_level: bool,
) {
    let area_state = game_data.area_state.mutate();

    if !game_data.area_blueprint.specs.disable_shards
        && (area_state.area_level > area_state.max_area_level_ever)
        && (area_state.area_level - game_data.area_blueprint.specs.starting_level + 1)
            .is_multiple_of(10)
    {
        game_data.player_resources.mutate().shards += 1.0;
    }

    game_data.player_specs.mutate().max_area_level =
        game_data.player_specs.read().max_area_level.max(
            area_state
                .area_level
                .saturating_add(game_data.area_blueprint.specs.item_level_modifier),
        );

    let new_max = area_state.area_level > area_state.max_area_level;

    if new_max {
        game_data.game_stats.elapsed_time_at_max_level = game_data.game_stats.elapsed_time;
        area_state.max_area_level = area_state.area_level;
        area_state.max_area_level_ever = area_state.max_area_level_ever.max(area_state.area_level);
    }

    match loot_generator::generate_loot(
        &game_data.area_blueprint.loot_table,
        &master_store.items_store,
        &master_store.item_affixes_table,
        &master_store.item_adjectives_table,
        &master_store.item_nouns_table,
        area_level.saturating_add(game_data.area_blueprint.specs.item_level_modifier),
        is_boss_level,
        new_max, // Only drop unique when new area completed
        None,
        area_state.loot_rarity,
    ) {
        Some(item_specs) => {
            for item_specs in loot_controller::drop_loot(
                &game_data.player_controller,
                game_data.queued_loot.mutate(),
                item_specs,
            ) {
                player_controller::sell_item(
                    &game_data.area_blueprint.specs,
                    game_data.player_specs.read(),
                    game_data.player_resources.mutate(),
                    &item_specs,
                );
            }
        }
        None => tracing::warn!("Failed to generate loot"),
    }

    area_state.waves_done = 1;
    if area_state.auto_progress {
        area_state.area_level += 1;
    }

    game_data.game_stats.areas_completed += 1;
}

fn handle_wave_completed_event(
    trigger_contexts: &mut Vec<TriggerContext>,
    events_queue: &mut EventsQueue,
    game_data: &mut GameInstanceData,
    area_level: AreaLevel,
) {
    let area_state = game_data.area_state.mutate();

    if !area_state.is_boss {
        area_state.waves_done += 1;
    }

    if area_state.is_boss || area_state.waves_done > WAVES_PER_AREA_LEVEL {
        events_queue.register_event(GameEvent::AreaCompleted {
            area_level,
            is_boss: area_state.is_boss,
        });
    }

    for triggered_effects in game_data
        .player_specs
        .read()
        .character_specs
        .triggers
        .iter()
    {
        if let EventTrigger::OnWaveCompleted = triggered_effects.trigger {
            trigger_contexts.push(TriggerContext {
                trigger: triggered_effects.clone(),
                source: CharacterId::Player,
                target: CharacterId::Player,
                hit_context: None,
                status_context: None,
                level: area_level as usize,
            });
        }
    }
}

fn handle_threat_increased_event(
    trigger_contexts: &mut Vec<TriggerContext>,
    game_data: &mut GameInstanceData,
    threat_level: ThreatLevel,
) {
    // To force recompute specs with new threat level, applying threat level stat converters
    for monster_state in game_data.monster_states.iter_mut() {
        monster_state.character_state.dirty_specs = true;
    }
    game_data.player_state.character_state.dirty_specs = true;

    for triggered_effects in game_data
        .player_specs
        .read()
        .character_specs
        .triggers
        .iter()
    {
        if let EventTrigger::OnThreatIncreased = triggered_effects.trigger {
            trigger_contexts.push(TriggerContext {
                trigger: triggered_effects.clone(),
                source: CharacterId::Player,
                target: CharacterId::Player,
                hit_context: None,
                status_context: None,
                level: threat_level as usize,
            });
        }
    }
}
