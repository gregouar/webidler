use std::{collections::HashMap, path::PathBuf};

use anyhow::Result;
use rand::Rng;

use shared::data::{monster::MonsterSpecs, world::WorldState};

use crate::game::world::{MonsterWaveBlueprint, WorldBlueprint};

use super::increase_factors::exponential_factor;

const MAX_MONSTERS: usize = 6; // TODO: Move

pub fn generate_monsters_wave_specs(
    world_blueprint: &WorldBlueprint,
    world_state: &WorldState,
) -> Result<Vec<MonsterSpecs>> {
    pick_wave(filter_waves(&world_blueprint.schema.waves, world_state))?
        .map(|wave| generate_all_monsters_specs(wave, &world_blueprint.monster_specs, world_state))
        .unwrap_or(Err(anyhow::format_err!("no monster wave available")))
}

fn filter_waves<'a>(
    waves: &'a Vec<MonsterWaveBlueprint>,
    world_state: &WorldState,
) -> Vec<&'a MonsterWaveBlueprint> {
    waves
        .iter()
        .filter(|wave| {
            world_state.area_level >= wave.min_level.unwrap_or(u16::MIN)
                && world_state.area_level <= wave.max_level.unwrap_or(u16::MAX)
        })
        .collect()
}

fn pick_wave<'a>(waves: Vec<&'a MonsterWaveBlueprint>) -> Result<Option<&'a MonsterWaveBlueprint>> {
    let mut rng = rand::rng();

    let total_probability = waves.iter().map(|w| w.probability).sum();
    if total_probability <= 0.0 {
        return Err(anyhow::format_err!(
            "no monsters wave probability available"
        ));
    }

    let p: f64 = rng.random_range(0.0..total_probability);
    Ok(waves
        .iter()
        .scan(0.0, |cumul_prob, w| {
            *cumul_prob += w.probability;
            Some((*cumul_prob, w))
        })
        .find(|(max_prob, w)| p >= *max_prob - w.probability && p < *max_prob)
        .map(|(_, w)| *w))
}

fn generate_all_monsters_specs(
    wave: &MonsterWaveBlueprint,
    monster_specs_blueprint: &HashMap<PathBuf, MonsterSpecs>,
    world_state: &WorldState,
) -> Result<Vec<MonsterSpecs>> {
    let mut rng = rand::rng();

    let mut monsters_specs = Vec::with_capacity(MAX_MONSTERS);
    'spawnloop: for spawn in wave.spawns.iter() {
        if spawn.max_quantity < spawn.min_quantity {
            return Err(anyhow::format_err!(
                "monster wave max_quantity below min_quantity"
            ));
        }
        for _ in 0..rng.random_range(spawn.min_quantity..=spawn.max_quantity) {
            if let Some(specs) = monster_specs_blueprint.get(&spawn.path) {
                monsters_specs.push(generate_monster_specs(specs, world_state));
                if monsters_specs.len() >= MAX_MONSTERS {
                    break 'spawnloop;
                }
            }
        }
    }
    Ok(monsters_specs)
}

fn generate_monster_specs(bp_specs: &MonsterSpecs, world_state: &WorldState) -> MonsterSpecs {
    let mut monster_specs = bp_specs.clone();
    let increase_factor = exponential_factor(world_state.area_level as f64);
    monster_specs.power_factor *= increase_factor;
    monster_specs.character_specs.max_health *= increase_factor;
    for skill_specs in monster_specs.character_specs.skill_specs.iter_mut() {
        skill_specs.min_damages *= increase_factor;
        skill_specs.max_damages *= increase_factor;
    }
    monster_specs
}
