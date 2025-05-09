use std::{collections::HashMap, path::PathBuf};

use anyhow::Result;

use shared::data::world::AreaLevel;
use shared::data::{monster::MonsterSpecs, world::WorldState};

use crate::game::data::world::{MonsterWaveBlueprint, WorldBlueprint};
use crate::game::utils::{increase_factors, rng};

const MAX_MONSTERS_PER_ROW: usize = 3; // TODO: Move

pub fn generate_monsters_wave_specs(
    world_blueprint: &WorldBlueprint,
    world_state: &WorldState,
) -> Result<Vec<MonsterSpecs>> {
    pick_wave(filter_waves(&world_blueprint.schema.waves, world_state))
        .map(|wave| {
            Ok(generate_all_monsters_specs(
                wave,
                &world_blueprint.monster_specs,
                world_state,
            ))
        })
        .unwrap_or(Err(anyhow::format_err!("no monster wave available")))
}

fn filter_waves<'a>(
    waves: &'a Vec<MonsterWaveBlueprint>,
    world_state: &WorldState,
) -> Vec<&'a MonsterWaveBlueprint> {
    waves
        .iter()
        .filter(|wave| {
            world_state.area_level >= wave.min_level.unwrap_or(AreaLevel::MIN)
                && world_state.area_level <= wave.max_level.unwrap_or(AreaLevel::MAX)
        })
        .collect()
}

fn pick_wave<'a>(waves: Vec<&'a MonsterWaveBlueprint>) -> Option<&'a MonsterWaveBlueprint> {
    rng::random_range(0.0..waves.iter().map(|w| w.probability).sum()).and_then(|p| {
        waves
            .iter()
            .scan(0.0, |cumul_prob, &w| {
                *cumul_prob += w.probability;
                Some((*cumul_prob, w))
            })
            .find(|(max_prob, w)| p >= *max_prob - w.probability && p < *max_prob)
            .map(|(_, w)| w)
    })
}

fn generate_all_monsters_specs(
    wave: &MonsterWaveBlueprint,
    monster_specs_blueprint: &HashMap<PathBuf, MonsterSpecs>,
    world_state: &WorldState,
) -> Vec<MonsterSpecs> {
    let mut top_space_available = MAX_MONSTERS_PER_ROW;
    let mut bot_space_available = MAX_MONSTERS_PER_ROW;

    let mut monsters_specs = Vec::with_capacity(top_space_available + bot_space_available);
    'spawnloop: for spawn in wave.spawns.iter() {
        for _ in 0..rng::random_range(spawn.min_quantity..=spawn.max_quantity).unwrap_or_default() {
            if let Some(specs) = monster_specs_blueprint.get(&spawn.path) {
                let (x_size, y_size) = specs.character_specs.size.get_xy_size();
                let use_top = y_size > 1 || top_space_available >= bot_space_available;
                let x_pos = (MAX_MONSTERS_PER_ROW + 1
                    - if use_top {
                        top_space_available
                    } else {
                        bot_space_available
                    }) as u8;

                if y_size > 1 {
                    if top_space_available >= x_size && bot_space_available >= x_size {
                        top_space_available -= x_size;
                        bot_space_available -= x_size;
                    } else {
                        continue;
                    }
                } else {
                    let row_to_use = if use_top {
                        &mut top_space_available
                    } else {
                        &mut bot_space_available
                    };
                    if *row_to_use >= x_size {
                        *row_to_use -= x_size
                    } else {
                        continue;
                    }
                }

                let mut specs = generate_monster_specs(specs, world_state);
                specs.character_specs.position_y = if use_top { 1 } else { 2 };
                specs.character_specs.position_x = x_pos;
                monsters_specs.push(specs);

                if top_space_available == 0 && bot_space_available == 0 {
                    break 'spawnloop;
                }
            }
        }
    }
    monsters_specs
}

fn generate_monster_specs(bp_specs: &MonsterSpecs, world_state: &WorldState) -> MonsterSpecs {
    let mut monster_specs = bp_specs.clone();
    let exp_factor = increase_factors::exponential(world_state.area_level as f64);
    let lin_factor = increase_factors::linear(world_state.area_level as f64);
    monster_specs.power_factor *= exp_factor;
    monster_specs.character_specs.max_health *= exp_factor;
    for skill_effect in monster_specs
        .skill_specs
        .iter_mut()
        .flat_map(|skill| skill.effects.iter_mut())
    {
        skill_effect.increase_effect(lin_factor);
    }
    monster_specs
}
