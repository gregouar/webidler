use anyhow::Result;

use shared::{
    computations,
    constants::{CHAMPION_LEVEL_INC, MONSTERS_DEFAULT_DAMAGE_INCREASE, MONSTER_INCREASE_FACTOR},
    data::{
        area::{AreaLevel, AreaState, AreaThreat},
        character::CharacterId,
        character_status::StatusSpecs,
        monster::{MonsterRarity, MonsterSpecs, MonsterState},
        passive::StatEffect,
        skill::SkillEffectType,
        stat_effect::{EffectsMap, Modifier, StatType},
    },
};

use crate::game::{
    data::{
        area::{AreaBlueprint, BossBlueprint, MonsterWaveBlueprint, MonsterWaveSpawnBlueprint},
        master_store::MonstersSpecsStore,
        monster::BaseMonsterSpecs,
        DataInit,
    },
    systems::{characters_updater, stats_updater},
    utils::rng::{self, RandomWeighted, Rollable},
};

use super::skills_updater;

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

/// Return generated monsters
pub fn generate_monsters_wave(
    area_blueprint: &AreaBlueprint,
    area_state: &mut AreaState,
    area_threat: &AreaThreat,
    area_effects: &EffectsMap,
    monsters_specs_store: &MonstersSpecsStore,
) -> Result<(Vec<MonsterSpecs>, Vec<MonsterState>)> {
    let (monster_specs, is_boss) = generate_monsters_wave_specs(
        area_blueprint,
        area_state,
        area_threat,
        area_effects,
        monsters_specs_store,
    )?;
    let monster_states = monster_specs.iter().map(MonsterState::init).collect();
    area_state.is_boss = is_boss;
    Ok((monster_specs, monster_states))
}

/// Return generated monsters + if it is boss
fn generate_monsters_wave_specs(
    area_blueprint: &AreaBlueprint,
    area_state: &mut AreaState,
    area_threat: &AreaThreat,
    area_effects: &EffectsMap,
    monsters_specs_store: &MonstersSpecsStore,
) -> Result<(Vec<MonsterSpecs>, bool)> {
    // Can only fight boss once per level
    if area_state.max_area_level < area_state.area_level {
        let available_bosses: Vec<_> = area_blueprint
            .bosses
            .iter()
            .filter(|b| {
                area_state.area_level >= b.level
                    && (area_state.area_level - b.level)
                        .is_multiple_of(b.interval.unwrap_or(AreaLevel::MAX))
            })
            .collect();

        if let Some(boss) = rng::random_weighted_pick(&available_bosses) {
            return Ok((
                generate_all_monsters_specs(
                    &boss.spawns,
                    area_blueprint,
                    area_state,
                    monsters_specs_store,
                    area_threat,
                    area_effects,
                ),
                true,
            ));
        }
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
                area_threat,
                area_effects,
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
    area_threat: &AreaThreat,
    area_effects: &EffectsMap,
) -> Vec<MonsterSpecs> {
    let mut grid = [[true; 3]; 2];
    let mut monsters = Vec::with_capacity(6);

    for spawn in spawns {
        let Some(template) = monsters_specs_store.get(&spawn.monster) else {
            tracing::error!("missing monster specs '{:?}'", spawn.monster);
            continue;
        };

        for _ in 0..spawn.quantity.roll() {
            if let Some((x, y)) = find_free_slot(&grid, template.character_specs.size.get_xy_size())
            {
                occupy_space(
                    &mut grid,
                    (x, y),
                    template.character_specs.size.get_xy_size(),
                );

                let mut specs = generate_monster_specs(
                    CharacterId::Monster(monsters.len()),
                    template,
                    area_blueprint,
                    area_state,
                    area_threat,
                    area_effects,
                );
                specs.character_specs.position_x = (x + 1) as u8;
                specs.character_specs.position_y = (y + 1) as u8;
                monsters.push(specs);

                // Early exit if grid is full
                if grid.iter().flat_map(|row| row.iter()).all(|cell| !cell) {
                    return monsters;
                }
            }
        }
    }

    monsters
}

fn find_free_slot(
    grid: &[[bool; 3]; 2],
    (x_size, y_size): (usize, usize),
) -> Option<(usize, usize)> {
    itertools::iproduct!(0..3, 0..2).find(|&(x, y)| {
        itertools::iproduct!(0..x_size, 0..y_size).all(|(dx, dy)| {
            grid.get(y + dy)
                .and_then(|row| row.get(x + dx))
                .copied()
                .unwrap_or_default()
        })
    })
}

fn occupy_space(
    grid: &mut [[bool; 3]; 2],
    (x, y): (usize, usize),
    (x_size, y_size): (usize, usize),
) {
    for (dx, dy) in itertools::iproduct!(0..x_size, 0..y_size) {
        if let Some(cell) = grid.get_mut(y + dy).and_then(|row| row.get_mut(x + dx)) {
            *cell = false;
        }
    }
}

fn generate_monster_specs(
    monster_id: CharacterId,
    bp_specs: &BaseMonsterSpecs,
    area_blueprint: &AreaBlueprint,
    area_state: &mut AreaState,
    area_threat: &AreaThreat,
    area_effects: &EffectsMap,
) -> MonsterSpecs {
    let mut monster_specs = MonsterSpecs::init(bp_specs.clone());
    let mut monster_level = area_state.area_level;

    monster_specs.character_specs.effects = area_effects.clone();

    if monster_specs.rarity == MonsterRarity::Normal
        && rng::random_range(0.0..=1.0).unwrap_or(1.0) < computations::gem_chance(area_state)
    {
        // area_state.last_champion_spawn = area_state.area_level;
        monster_specs.rarity = MonsterRarity::Champion;
        monster_level += CHAMPION_LEVEL_INC;
    };

    let exp_factor = computations::exponential(monster_level, MONSTER_INCREASE_FACTOR);
    let reward_factor = computations::exponential(
        monster_level.saturating_sub(area_blueprint.specs.starting_level) + 1,
        MONSTER_INCREASE_FACTOR,
    );

    monster_specs.power_factor *= exp_factor;
    monster_specs.reward_factor *= reward_factor;
    monster_specs.character_specs.max_life *= exp_factor;

    // Apply upgrade effects
    let upgrade_effects = [StatEffect {
        stat: StatType::Damage {
            skill_type: None,
            damage_type: None,
        },
        modifier: Modifier::Multiplier,
        value: (monster_level as f64 - 1.0) * MONSTERS_DEFAULT_DAMAGE_INCREASE,
        bypass_ignore: true,
    }];
    for skill_specs in monster_specs.skill_specs.iter_mut() {
        if skill_specs.base.upgrade_effects.is_empty() {
            skills_updater::update_skill_specs(
                skill_specs,
                upgrade_effects.iter(),
                None,
                area_threat,
            );
        } else {
            let effects: Vec<_> =
                (&skills_updater::compute_skill_upgrade_effects(skill_specs, monster_level)).into();
            skills_updater::update_skill_specs(skill_specs, effects.iter(), None, area_threat);
        }

        // Link monster_id to triggers of skills
        for trigger in skill_specs.triggers.iter_mut() {
            trigger.triggered_effect.owner = Some(monster_id);
        }

        for effect in skill_specs
            .targets
            .iter_mut()
            .flat_map(|target| target.effects.iter_mut())
        {
            if let SkillEffectType::ApplyStatus {
                ref mut statuses, ..
            } = effect.effect_type
            {
                for status in statuses {
                    if let StatusSpecs::Trigger(ref mut trigger_specs) = status.status_type {
                        trigger_specs.triggered_effect.owner = Some(monster_id);
                    }
                }
            }
        }
    }

    // Apply area effects
    let area_effects = stats_updater::stats_map_to_vec(&area_effects, area_threat);
    monster_specs.character_specs =
        characters_updater::update_character_specs(&monster_specs.character_specs, &area_effects);
    // monster_specs.character_specs.effects = effects_map;
    for skill_specs in monster_specs.skill_specs.iter_mut() {
        skills_updater::apply_effects_to_skill_specs(skill_specs, area_effects.iter());
    }

    monster_specs
}
