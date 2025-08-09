use shared::{
    constants::WAVES_PER_AREA_LEVEL,
    data::{area::AreaLevel, character::CharacterId, trigger::EventTrigger},
};

use crate::game::{
    data::{
        event::{EventsQueue, GameEvent, HitEvent},
        master_store::MasterStore,
    },
    game_data::GameInstanceData,
    systems::triggers_controller,
};

use super::{
    loot_controller, loot_generator, player_controller, triggers_controller::TriggerContext,
};

pub async fn resolve_events(
    events_queue: &mut EventsQueue,
    game_data: &mut GameInstanceData,
    master_store: &MasterStore,
) {
    let mut trigger_effects = Vec::new();

    let events = events_queue.consume_events();
    for event in events.iter() {
        match event {
            GameEvent::Hit(hit_event) => {
                handle_hit_event(&mut trigger_effects, game_data, hit_event)
            }
            GameEvent::Kill { target } => {
                handle_kill_event(&mut trigger_effects, game_data, *target)
            }
            GameEvent::AreaCompleted(area_level) => {
                handle_area_completed_event(game_data, master_store, *area_level)
            }
            GameEvent::WaveCompleted(area_level) => handle_wave_completed_event(
                &mut trigger_effects,
                events_queue,
                game_data,
                *area_level,
            ),
        }
    }

    triggers_controller::apply_trigger_effects(
        events_queue,
        game_data,
        master_store,
        trigger_effects,
    );
}

fn handle_hit_event<'a>(
    trigger_effects: &mut Vec<TriggerContext<'a>>,
    game_data: &mut GameInstanceData,
    hit_event: &'a HitEvent,
) {
    // TODO: Have the same for monsters... might need to go for an actual ECS for that
    for triggered_effects in game_data.player_specs.read().triggers.iter() {
        match triggered_effects.trigger {
            EventTrigger::OnHit(_) if hit_event.source == CharacterId::Player => {}
            EventTrigger::OnTakeHit(_) if hit_event.target == CharacterId::Player => {}
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
        {
            trigger_effects.push(TriggerContext {
                trigger: triggered_effects.clone(),
                source: hit_event.source,
                target: hit_event.target,
                hit_context: Some(hit_event),
                area_level: game_data.area_state.read().area_level,
            });
        }
    }
}

fn handle_kill_event(
    trigger_effects: &mut Vec<TriggerContext>,
    game_data: &mut GameInstanceData,
    target: CharacterId,
) {
    match target {
        CharacterId::Monster(monster_index) => {
            game_data.game_stats.monsters_killed += 1;
            if let Some(monster_specs) = game_data.monster_specs.get(monster_index) {
                let gold_reward = player_controller::reward_player(
                    game_data.player_resources.mutate(),
                    game_data.player_specs.read(),
                    monster_specs,
                );
                if let Some(monster_state) = game_data.monster_states.get_mut(monster_index) {
                    monster_state.gold_reward = gold_reward;
                }
            }

            for triggered_effects in game_data.player_specs.read().triggers.iter() {
                if let EventTrigger::OnKill = triggered_effects.trigger {
                    trigger_effects.push(TriggerContext {
                        trigger: triggered_effects.clone(),
                        source: CharacterId::Player,
                        target,
                        hit_context: None,
                        area_level: game_data.area_state.read().area_level,
                    });
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
) {
    match loot_generator::generate_loot(
        area_level,
        &game_data.area_blueprint.loot_table,
        &master_store.items_store,
        &master_store.item_affixes_table,
        &master_store.item_adjectives_table,
        &master_store.item_nouns_table,
    ) {
        Some(item_specs) => {
            for item_specs in loot_controller::drop_loot(
                &game_data.player_controller,
                game_data.queued_loot.mutate(),
                item_specs,
            ) {
                player_controller::sell_item(
                    game_data.player_specs.read(),
                    game_data.player_resources.mutate(),
                    &item_specs,
                );
            }
        }
        None => tracing::warn!("Failed to generate loot"),
    }

    let area_state = game_data.area_state.mutate();
    area_state.waves_done = 1;
    if area_state.auto_progress {
        area_state.area_level += 1;
    }

    game_data.game_stats.areas_completed += 1;
    game_data.game_stats.highest_area_level =
        game_data.game_stats.highest_area_level.max(area_level);
}

fn handle_wave_completed_event(
    trigger_effects: &mut Vec<TriggerContext>,
    events_queue: &mut EventsQueue,
    game_data: &mut GameInstanceData,
    area_level: AreaLevel,
) {
    let area_state = game_data.area_state.mutate();

    if !area_state.is_boss {
        area_state.waves_done += 1;
    }

    if area_state.is_boss || area_state.waves_done > WAVES_PER_AREA_LEVEL {
        events_queue.register_event(GameEvent::AreaCompleted(area_level));
    }

    for triggered_effects in game_data.player_specs.read().triggers.iter() {
        if let EventTrigger::OnWaveCompleted = triggered_effects.trigger {
            trigger_effects.push(TriggerContext {
                trigger: triggered_effects.clone(),
                source: CharacterId::Player,
                target: CharacterId::Player,
                hit_context: None,
                area_level: game_data.area_state.read().area_level,
            });
        }
    }
}
