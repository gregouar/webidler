use anyhow::Result;
use std::time::Duration;

use shared::{
    constants::{THREAT_BOSS_COOLDOWN, THREAT_COOLDOWN},
    data::{area::AreaThreat, character::CharacterId, player::PlayerState},
};

use crate::game::systems::benedictions_controller;

use super::{
    data::{
        DataInit,
        event::{EventsQueue, GameEvent},
        master_store::MasterStore,
    },
    game_data::GameInstanceData,
    systems::{
        area_controller, events_resolver, monsters_controller, monsters_updater, monsters_wave,
        player_updater,
    },
    utils::LazySyncer,
};

const PLAYER_RESPAWN_PERIOD: Duration = Duration::from_secs(5);

pub async fn reset_entities(game_data: &mut GameInstanceData) {
    if game_data.end_quest {
        return;
    }

    player_updater::reset_player(&mut game_data.player_state);
    monsters_updater::reset_monsters(&mut game_data.monster_states);
}

pub async fn tick(
    events_queue: &mut EventsQueue,
    game_data: &mut GameInstanceData,
    master_store: &MasterStore,
    elapsed_time: Duration,
) -> Result<()> {
    if game_data.end_quest {
        return Ok(());
    }

    update_threat(events_queue, game_data, elapsed_time);

    if game_data.area_state.read().rush_mode {
        game_data.player_stamina = game_data.player_stamina.saturating_sub(elapsed_time);
        if game_data.player_stamina.is_zero() {
            game_data.area_state.mutate().rush_mode = false;
        }
    }

    // If client input altered the player specs (equip item, ...), we need to recompute the currents specs
    if game_data.player_specs.need_to_sync()
        || game_data.player_inventory.need_to_sync()
        || game_data.passives_tree_state.need_to_sync()
        || game_data.player_state.character_state.dirty_specs
    {
        // This feels so dirty =(
        game_data.player_state.character_state.dirty_specs = false;

        player_updater::update_player_specs(
            game_data.player_specs.mutate(),
            &game_data.player_state,
            game_data.player_inventory.read(),
            &game_data.passives_tree_specs,
            game_data.passives_tree_state.read(),
            &benedictions_controller::generate_effects_map_from_benedictions(
                &master_store.benedictions_store,
                &game_data.player_benedictions,
            ),
            &game_data.area_threat,
        );
    }

    if game_data
        .monster_states
        .iter()
        .any(|m| m.character_state.dirty_specs)
    {
        for ((base_specs, monster_specs), monster_state) in game_data
            .monster_base_specs
            .read()
            .iter()
            .zip(game_data.monster_specs.iter_mut())
            .zip(game_data.monster_states.iter_mut())
        {
            if monster_state.character_state.dirty_specs {
                monster_state.character_state.dirty_specs = false;
                monsters_updater::update_monster_specs(
                    base_specs,
                    monster_specs,
                    monster_state,
                    &game_data.area_threat,
                );
            }
        }
    }

    control_entities(events_queue, game_data, master_store).await?;
    events_resolver::resolve_events(events_queue, game_data, master_store).await;
    update_entities(events_queue, game_data, elapsed_time).await;

    game_data.game_stats.elapsed_time += elapsed_time;
    Ok(())
}

async fn control_entities(
    events_queue: &mut EventsQueue,
    game_data: &mut GameInstanceData,
    master_store: &MasterStore,
) -> Result<()> {
    if !game_data.player_state.character_state.is_alive {
        game_data.area_threat.cooldown = 0.0;
        game_data.monster_wave_delay =
            Duration::from_secs_f32(game_data.player_specs.read().movement_cooldown);

        if game_data.player_respawn_delay.is_zero() {
            respawn_player(master_store, game_data);
        }
        return Ok(());
    }

    game_data.player_respawn_delay = PLAYER_RESPAWN_PERIOD;

    let monsters_exist = !game_data.monster_specs.is_empty();
    let mut monsters_still_alive: Vec<_> = game_data
        .monster_specs
        .iter()
        .zip(game_data.monster_states.iter_mut())
        .enumerate()
        .filter(|(_, (_, m))| m.character_state.is_alive)
        .map(|(i, (x, y))| {
            (
                CharacterId::Monster(i),
                (&x.character_specs, &mut y.character_state),
            )
        })
        .collect();

    game_data.player_controller.control_player(
        events_queue,
        game_data.player_specs.read(),
        &mut game_data.player_state,
        &mut monsters_still_alive,
    );

    let wave_completed = monsters_still_alive.is_empty();
    if wave_completed || game_data.area_state.read().going_back > 0 {
        game_data.area_threat.cooldown = 0.0;
        if wave_completed
            && !game_data.wave_completed
            && monsters_exist
            && game_data.area_state.read().going_back == 0
        {
            game_data.wave_completed = true;
            events_queue.register_event(GameEvent::WaveCompleted(
                game_data.area_state.read().area_level,
            ));
        }

        if game_data.monster_wave_delay.is_zero() {
            if game_data.area_state.read().going_back > 0 {
                let area_state = game_data.area_state.mutate();
                let amount = area_state.going_back;
                area_controller::decrease_area_level(
                    &game_data.area_blueprint.specs,
                    area_state,
                    amount,
                );
                area_state.going_back = 0;
            }

            let (monster_specs, monster_states) = monsters_wave::generate_monsters_wave(
                &game_data.area_blueprint,
                game_data.area_state.mutate(),
                &game_data.area_threat,
                &game_data.area_effects,
                &master_store.monster_specs_store,
            )?;
            game_data.monster_base_specs = LazySyncer::new(monster_specs.clone());
            game_data.monster_specs = monster_specs;
            game_data.monster_states = monster_states;

            game_data.area_threat = AreaThreat {
                threat_level: 0,
                cooldown: if game_data.area_state.read().is_boss {
                    THREAT_BOSS_COOLDOWN
                } else {
                    THREAT_COOLDOWN
                },
                elapsed_cooldown: 0.0,
                just_increased: false,
            };

            game_data.wave_completed = false;
        }
    } else {
        game_data.monster_wave_delay =
            Duration::from_secs_f32(game_data.player_specs.read().movement_cooldown);
        monsters_controller::control_monsters(
            events_queue,
            &game_data.monster_specs,
            &mut game_data.monster_states,
            game_data.player_specs.read(),
            &mut game_data.player_state,
        );
    }

    Ok(())
}

pub fn update_threat(
    events_queue: &mut EventsQueue,
    game_data: &mut GameInstanceData,
    elapsed_time: Duration,
) {
    game_data.area_threat.just_increased = false;
    if game_data.area_threat.cooldown > 0.0 {
        game_data.area_threat.elapsed_cooldown +=
            elapsed_time.as_secs_f32() * game_data.player_specs.read().threat_gain * 0.01
                / game_data.area_threat.cooldown;
        if game_data.area_threat.elapsed_cooldown >= 1.0 {
            game_data.area_threat.elapsed_cooldown -= 1.0;
            game_data.area_threat.threat_level =
                game_data.area_threat.threat_level.saturating_add(1);
            game_data.area_threat.just_increased = true;
            events_queue.register_event(GameEvent::ThreatIncreased(
                game_data.area_threat.threat_level,
            ));
        }
    }
}

async fn update_entities(
    events_queue: &mut EventsQueue,
    game_data: &mut GameInstanceData,
    elapsed_time: Duration,
) {
    game_data.player_respawn_delay = game_data.player_respawn_delay.saturating_sub(elapsed_time);
    game_data.monster_wave_delay = game_data.monster_wave_delay.saturating_sub(elapsed_time);

    if !game_data.player_state.character_state.is_alive
        || game_data.area_state.read().going_back > 0
    {
        return;
    }

    player_updater::update_player_state(
        events_queue,
        elapsed_time,
        game_data.player_specs.read(),
        &mut game_data.player_state,
    );
    monsters_updater::update_monster_states(
        events_queue,
        elapsed_time,
        &game_data.monster_specs,
        &mut game_data.monster_states,
    );
}

fn respawn_player(master_store: &MasterStore, game_data: &mut GameInstanceData) {
    game_data.monster_base_specs.mutate().clear();
    game_data.monster_specs.clear();
    game_data.monster_states.clear();

    player_updater::update_player_specs(
        game_data.player_specs.mutate(),
        &game_data.player_state,
        game_data.player_inventory.read(),
        &game_data.passives_tree_specs,
        game_data.passives_tree_state.read(),
        &benedictions_controller::generate_effects_map_from_benedictions(
            &master_store.benedictions_store,
            &game_data.player_benedictions,
        ),
        &game_data.area_threat,
    );

    game_data.player_state = PlayerState::init(game_data.player_specs.read());
    // for skill_state in game_data.player_state.skills_states.iter_mut() {
    //     skill_state.elapsed_cooldown = 0.5;
    // }

    if game_data.area_state.read().auto_progress {
        area_controller::decrease_area_level(
            &game_data.area_blueprint.specs,
            game_data.area_state.mutate(),
            1,
        );
    } else {
        game_data.area_state.mutate().waves_done = 1;
    }
}
