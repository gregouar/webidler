use anyhow::Result;

use shared::data::{
    area::{AreaLevel, AreaState},
    monster::{MonsterRarity, MonsterSpecs, MonsterState},
    passive::StatEffect,
    stat_effect::{Modifier, StatType},
};

use crate::{
    constants::{CHAMPION_BASE_CHANCES, CHAMPION_INC_CHANCES, CHAMPION_LEVEL_INC},
    game::{
        data::{
            area::{AreaBlueprint, BossBlueprint, MonsterWaveBlueprint, MonsterWaveSpawnBlueprint},
            master_store::MonstersSpecsStore,
            monster::BaseMonsterSpecs,
            DataInit,
        },
        utils::{
            increase_factors,
            rng::{self, RandomWeighted},
        },
    },
};

use super::skills_updater;

const MAX_MONSTERS_PER_ROW: usize = 3; // TODO: Move

impl RandomWeighted for &MonsterWaveBlueprint {
    fn random_weight(&self) -> u64 {
        self.weight
    }
}

impl RandomWeighted for &BossBlueprint {
    fn random_weight(&self) -> u64 {
        1
    }
}

/// Return generated monsters + if it is boss
pub fn generate_monsters_wave(
    area_blueprint: &AreaBlueprint,
    area_state: &mut AreaState,
    monsters_specs_store: &MonstersSpecsStore,
) -> Result<(Vec<MonsterSpecs>, Vec<MonsterState>, bool)> {
    let (monster_specs, is_boss) =
        generate_monsters_wave_specs(area_blueprint, area_state, monsters_specs_store)?;
    let monster_states = monster_specs.iter().map(MonsterState::init).collect();
    Ok((monster_specs, monster_states, is_boss))
}

/// Return generated monsters + if it is boss
fn generate_monsters_wave_specs(
    area_blueprint: &AreaBlueprint,
    area_state: &mut AreaState,
    monsters_specs_store: &MonstersSpecsStore,
) -> Result<(Vec<MonsterSpecs>, bool)> {
    let available_bosses: Vec<_> = area_blueprint
        .bosses
        .iter()
        .filter(|b| {
            area_state.area_level >= b.level
                && (area_state.area_level - b.level) % b.interval.unwrap_or(AreaLevel::MAX) == 0
        })
        .collect();

    if let Some(boss) = rng::random_weighted_pick(&available_bosses) {
        return Ok((
            generate_all_monsters_specs(
                &boss.spawns,
                area_blueprint,
                area_state,
                monsters_specs_store,
            ),
            true,
        ));
    }

    let available_waves: Vec<_> = area_blueprint
        .waves
        .iter()
        .filter(|wave| {
            area_state.area_level >= wave.min_level.unwrap_or(AreaLevel::MIN)
                && area_state.area_level <= wave.max_level.unwrap_or(AreaLevel::MAX)
        })
        .collect();

    if let Some(wave) = rng::random_weighted_pick(&available_waves) {
        return Ok((
            generate_all_monsters_specs(
                &wave.spawns,
                area_blueprint,
                area_state,
                monsters_specs_store,
            ),
            false,
        ));
    }

    Err(anyhow::format_err!("no monster wave available"))
}

fn generate_all_monsters_specs(
    spawns: &[MonsterWaveSpawnBlueprint],
    area_blueprint: &AreaBlueprint,
    area_state: &mut AreaState,
    monsters_specs_store: &MonstersSpecsStore,
) -> Vec<MonsterSpecs> {
    let mut top_space_available = MAX_MONSTERS_PER_ROW;
    let mut bot_space_available = MAX_MONSTERS_PER_ROW;

    let mut monsters_specs = Vec::with_capacity(top_space_available + bot_space_available);
    'spawnloop: for spawn in spawns.iter() {
        for _ in 0..rng::random_range(spawn.min_quantity..=spawn.max_quantity).unwrap_or_default() {
            if let Some(specs) = monsters_specs_store.get(&spawn.monster) {
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

                let mut specs = generate_monster_specs(specs, area_blueprint, area_state);
                specs.character_specs.position_y = if use_top { 1 } else { 2 };
                specs.character_specs.position_x = x_pos;
                monsters_specs.push(specs);

                if top_space_available == 0 && bot_space_available == 0 {
                    break 'spawnloop;
                }
            } else {
                tracing::error!("missing monster specs '{:?}'", spawn.monster);
            }
        }
    }
    monsters_specs
}

fn generate_monster_specs(
    bp_specs: &BaseMonsterSpecs,
    area_blueprint: &AreaBlueprint,
    area_state: &mut AreaState,
) -> MonsterSpecs {
    let mut monster_specs = MonsterSpecs::init(bp_specs.clone());
    let mut monster_level = area_state.area_level;

    if area_state.area_level > area_state.last_champion_spawn {
        let gem_chances = CHAMPION_BASE_CHANCES
            + (CHAMPION_INC_CHANCES
                * (area_state.area_level - area_state.last_champion_spawn) as f64);
        if rng::random_range(0.0..1.0).unwrap_or(1.0) <= gem_chances {
            // area_state.last_champion_spawn = area_state.area_level;
            monster_specs.rarity = MonsterRarity::Champion;
            monster_level += CHAMPION_LEVEL_INC;
        }
    };

    let exp_factor =
        increase_factors::exponential(monster_level, increase_factors::MONSTER_INCREASE_FACTOR);
    let reward_factor = increase_factors::exponential(
        monster_level - area_blueprint.specs.starting_level + 1,
        increase_factors::MONSTER_INCREASE_FACTOR,
    );

    monster_specs.power_factor *= exp_factor;
    monster_specs.reward_factor *= reward_factor;
    monster_specs.character_specs.max_life *= exp_factor;

    let effects = vec![StatEffect {
        stat: StatType::Damage {
            skill_type: None,
            damage_type: None,
        },
        modifier: Modifier::Multiplier,
        value: (monster_level as f64 - 1.0) / 10.0,
    }];
    for skill_specs in monster_specs.skill_specs.iter_mut() {
        if skill_specs.base.upgrade_effects.is_empty() {
            skills_updater::update_skill_specs(skill_specs, effects.iter(), None);
        } else {
            skills_updater::update_skill_specs(
                skill_specs,
                skills_updater::compute_skill_upgrade_effects(skill_specs, monster_level)
                    .collect::<Vec<_>>()
                    .iter(),
                None,
            );
        }
    }
    monster_specs
}
