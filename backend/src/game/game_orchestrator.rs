use anyhow::Result;
use std::time::{Duration, Instant};

use shared::data::{character::CharacterId, monster::MonsterState, player::PlayerState};

use super::{
    data::{
        event::{EventsQueue, GameEvent},
        master_store::MasterStore,
        DataInit,
    },
    game_data::GameInstanceData,
    systems::{
        loot_controller, loot_generator, monsters_controller, monsters_updater, monsters_wave,
        player_controller, player_updater, world_controller,
    },
    utils::LazySyncer,
};

const PLAYER_RESPAWN_PERIOD: Duration = Duration::from_secs(5);

const WAVES_PER_AREA_LEVEL: u8 = 5;

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
        player_controller::update_player_specs(
            game_data.player_specs.mutate(),
            game_data.player_inventory.read(),
            &game_data.passives_tree_specs,
            game_data.passives_tree_state.read(),
        );
    }

    control_entities(events_queue, game_data, master_store).await?;
    resolve_events(events_queue, game_data).await;
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
            game_data.game_stats.player_deaths += 1;
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
        game_data.player_controller.reset();

        if monsters_still_alive.is_empty() || game_data.world_state.read().going_back > 0 {
            if game_data.world_state.read().going_back == 0
                && !game_data.looted
                && game_data.world_state.read().waves_done == WAVES_PER_AREA_LEVEL
            {
                if let Some(item_specs) = loot_generator::generate_loot(
                    game_data.world_state.read().area_level,
                    &game_data.world_blueprint.loot_table,
                    &master_store.items_store,
                    &master_store.item_affixes_table,
                    &master_store.item_adjectives_table,
                    &master_store.item_nouns_table,
                ) {
                    loot_controller::drop_loot(game_data.queued_loot.mutate(), item_specs);
                }
                game_data.looted = true;
            }

            if game_data.monster_wave_delay.elapsed()
                > Duration::from_secs_f32(game_data.player_specs.read().movement_cooldown)
            {
                generate_monsters_wave(game_data, master_store).await?;
                game_data.looted = false;
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

async fn resolve_events(events_queue: &mut EventsQueue, game_data: &mut GameInstanceData) {
    for event in events_queue.consume_events() {
        // TODO
        match event {
            GameEvent::Hit(_) => {}
            GameEvent::CriticalStrike(_) => {}
            GameEvent::Block(_) => {}
            GameEvent::Kill { target } => {
                if let CharacterId::Monster(monster_index) = target {
                    game_data.game_stats.monsters_killed += 1;
                    if let Some(monster_specs) =
                        game_data.monster_specs.read().get(monster_index as usize)
                    {
                        player_controller::reward_player(
                            game_data.player_resources.mutate(),
                            game_data.player_specs.read(),
                            monster_specs,
                        );
                    }
                }
            }
        }
    }
}

async fn generate_monsters_wave(
    game_data: &mut GameInstanceData,
    master_store: &MasterStore,
) -> Result<()> {
    let world_state = game_data.world_state.mutate();

    if world_state.going_back > 0 {
        let amount = world_state.going_back;
        world_controller::decrease_area_level(world_state, amount);
    }

    world_state.going_back = 0;
    world_state.waves_done += 1;

    if world_state.waves_done > WAVES_PER_AREA_LEVEL {
        world_state.waves_done = 1;
        game_data.game_stats.areas_completed += 1;
        if world_state.auto_progress {
            world_state.area_level += 1;
        }
    }

    game_data.game_stats.highest_area_level = game_data
        .game_stats
        .highest_area_level
        .max(world_state.area_level);

    game_data.monster_specs = LazySyncer::new(monsters_wave::generate_monsters_wave_specs(
        &game_data.world_blueprint,
        world_state,
        &master_store.monster_specs_store,
    )?);

    game_data.monster_states = game_data
        .monster_specs
        .read()
        .iter()
        .map(MonsterState::init)
        .collect();

    Ok(())
}

fn respawn_player(game_data: &mut GameInstanceData) {
    game_data.monster_specs.mutate().clear();
    game_data.monster_states = Vec::new();

    game_data.player_state = PlayerState::init(game_data.player_specs.read());

    world_controller::decrease_area_level(game_data.world_state.mutate(), 1);
}
