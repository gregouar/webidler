use anyhow::Result;
use std::time::{Duration, Instant};

use shared::data::{character::CharacterId, player::PlayerState};

use super::{
    data::{
        event::{EventsQueue, GameEvent},
        master_store::MasterStore,
        DataInit,
    },
    game_data::GameInstanceData,
    systems::{
        events_resolver, monsters_controller, monsters_updater, monsters_wave, player_updater,
        world_controller,
    },
    utils::LazySyncer,
};

const PLAYER_RESPAWN_PERIOD: Duration = Duration::from_secs(5);

pub async fn reset_entities(game_data: &mut GameInstanceData) {
    player_updater::reset_player(&mut game_data.player_state);
    monsters_updater::reset_monsters(&mut game_data.monster_states);
}

pub async fn tick(
    events_queue: &mut EventsQueue,
    game_data: &mut GameInstanceData,
    master_store: &MasterStore,
    elapsed_time: Duration,
) -> Result<()> {
    // If client input altered the player specs (equip item, ...), we need to recompute the currents specs
    if game_data.player_specs.need_to_sync()
        || game_data.player_inventory.need_to_sync()
        || game_data.passives_tree_state.need_to_sync()
    {
        player_updater::update_player_specs(
            game_data.player_specs.mutate(),
            game_data.player_inventory.read(),
            &game_data.passives_tree_specs,
            game_data.passives_tree_state.read(),
        );
    }

    control_entities(events_queue, game_data, master_store).await?;
    events_resolver::resolve_events(events_queue, game_data, master_store).await;
    update_entities(events_queue, game_data, elapsed_time).await;

    Ok(())
}

async fn control_entities(
    events_queue: &mut EventsQueue,
    game_data: &mut GameInstanceData,
    master_store: &MasterStore,
) -> Result<()> {
    if !game_data.player_state.character_state.is_alive {
        if game_data.player_respawn_delay.elapsed() > PLAYER_RESPAWN_PERIOD {
            respawn_player(game_data);
        }
    } else {
        game_data.player_respawn_delay = Instant::now();

        let mut monsters_still_alive: Vec<_> = game_data
            .monster_specs
            .read()
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
        if wave_completed || game_data.world_state.read().going_back > 0 {
            if wave_completed && !game_data.wave_completed {
                game_data.wave_completed = true;
                events_queue.register_event(GameEvent::WaveCompleted(
                    game_data.world_state.read().area_level,
                ));
            }

            if game_data.monster_wave_delay.elapsed()
                > Duration::from_secs_f32(game_data.player_specs.read().movement_cooldown)
            {
                if game_data.world_state.read().going_back > 0 {
                    let world_state: &mut shared::data::world::WorldState =
                        game_data.world_state.mutate();
                    let amount = world_state.going_back;
                    world_controller::decrease_area_level(world_state, amount);
                    world_state.going_back = 0;
                }
                let (monster_specs, monster_states) = monsters_wave::generate_monsters_wave(
                    &game_data.world_blueprint,
                    game_data.world_state.read(),
                    &master_store.monster_specs_store,
                )?;
                game_data.monster_specs = LazySyncer::new(monster_specs);
                game_data.monster_states = monster_states;

                game_data.wave_completed = false;
            }
        } else {
            game_data.monster_wave_delay = Instant::now();
            monsters_controller::control_monsters(
                events_queue,
                game_data.monster_specs.read(),
                &mut game_data.monster_states,
                game_data.player_specs.read(),
                &mut game_data.player_state,
            );
        }
    }

    Ok(())
}

async fn update_entities(
    events_queue: &mut EventsQueue,
    game_data: &mut GameInstanceData,
    elapsed_time: Duration,
) {
    game_data.game_stats.elapsed_time += elapsed_time;
    player_updater::update_player_state(
        events_queue,
        elapsed_time,
        game_data.player_specs.read(),
        &mut game_data.player_state,
    );
    monsters_updater::update_monster_states(
        events_queue,
        elapsed_time,
        game_data.monster_specs.read(),
        &mut game_data.monster_states,
    );
}

fn respawn_player(game_data: &mut GameInstanceData) {
    game_data.monster_specs.mutate().clear();
    game_data.monster_states.clear();

    game_data.player_state = PlayerState::init(game_data.player_specs.read());

    world_controller::decrease_area_level(game_data.world_state.mutate(), 1);
}
