use std::{
    env, fs,
    path::{Path, PathBuf},
    time::Duration,
};

use anyhow::{Context, Result};
use backend::game::{
    data::{event::EventsQueue, master_store::MasterStore},
    game_data::GameInstanceData,
    game_orchestrator, game_sync,
    systems::{
        inventory_controller, loot_generator, player_controller,
        player_controller::PlayerController, player_updater, quests_controller,
        statuses_controller,
    },
};
use backend_shared::signature::HmacKey;
use shared::{
    data::{
        character_status::{StatusMap, StatusSpecs},
        item::ItemRarity,
        modifier::Modifier,
        passive::PassivesTreeState,
        player::{PlayerInventory, PlayerResources},
        realms::Realm,
        skill::{DamageType, SkillType},
        stat_effect::{StatSkillFilter, StatType},
    },
    messages::server::ServerMessage,
};

const DEFAULT_DATA_DIR: &str = "data";
const DEFAULT_OUTPUT: &str = "backend/.local/sync_zstd_dictionary.bin";
const DEFAULT_DICT_SIZE: usize = 16 * 1024;
const DEFAULT_TICKS_PER_SCENARIO: usize = 160;
const DEFAULT_SAMPLE_DIR: &str = "target/sync_zstd_samples";

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::from_args()?;
    let master_store = MasterStore::load_from_folder(&config.data_dir, HmacKey::default())
        .await
        .with_context(|| format!("failed to load data from '{}'", config.data_dir.display()))?;

    let samples = generate_samples(&master_store, config.ticks_per_scenario).await?;
    anyhow::ensure!(!samples.is_empty(), "no sync samples generated");

    if let Some(sample_dir) = &config.sample_dir {
        write_samples(sample_dir, &samples)?;
    }

    let dictionary = zstd::dict::from_samples(&samples, config.dict_size).with_context(|| {
        format!(
            "failed to train zstd dictionary from {} samples",
            samples.len()
        )
    })?;

    let total_sample_bytes: usize = samples.iter().map(Vec::len).sum();
    let compressed_without_dict = compressed_len_without_dict(&samples)?;
    let compressed_with_dict = compressed_len_with_dict(&samples, &dictionary)?;

    if !config.check_only {
        if let Some(parent) = config.output.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create '{}'", parent.display()))?;
        }
        fs::write(&config.output, &dictionary)
            .with_context(|| format!("failed to write '{}'", config.output.display()))?;
    }

    println!(
        "trained sync zstd dictionary: samples={}, raw={} bytes, dict={} bytes, zstd={} bytes, zstd+dict={} bytes, output={}",
        samples.len(),
        total_sample_bytes,
        dictionary.len(),
        compressed_without_dict,
        compressed_with_dict,
        if config.check_only {
            "(check only)".to_string()
        } else {
            config.output.display().to_string()
        }
    );

    Ok(())
}

async fn generate_samples(
    master_store: &MasterStore,
    ticks_per_scenario: usize,
) -> Result<Vec<Vec<u8>>> {
    let mut samples = Vec::new();
    let mut area_ids = master_store
        .area_blueprints_store
        .keys()
        .cloned()
        .collect::<Vec<_>>();
    area_ids.sort();

    for area_id in area_ids {
        for max_area_level in [0, 5, 25, 75] {
            let Ok(mut game_data) = init_game_data(master_store, area_id.clone(), max_area_level)
            else {
                continue;
            };

            game_data.reset_syncers();
            push_sync_sample(&mut samples, &mut game_data)?;

            let mut events_queue = EventsQueue::new();
            for tick in 0..ticks_per_scenario {
                game_orchestrator::reset_entities(&mut game_data).await;
                mutate_scenario(master_store, &mut game_data, tick);
                game_orchestrator::tick(
                    &mut events_queue,
                    &mut game_data,
                    master_store,
                    Duration::from_millis(100),
                )
                .await?;
                push_sync_sample(&mut samples, &mut game_data)?;
            }

            quests_controller::end_quest(master_store, &mut game_data);
            push_sync_sample(&mut samples, &mut game_data)?;
        }
    }

    Ok(samples)
}

fn init_game_data(
    master_store: &MasterStore,
    area_id: String,
    max_area_level: u16,
) -> Result<GameInstanceData> {
    let mut player_inventory = PlayerInventory {
        max_bag_size: 40,
        ..Default::default()
    };
    equip_sample_item(
        master_store,
        &mut player_inventory,
        "dagger",
        ItemRarity::Normal,
        0,
    );
    equip_sample_item(
        master_store,
        &mut player_inventory,
        "leather_armor",
        ItemRarity::Magic,
        max_area_level,
    );
    add_unique_item_samples(master_store, &mut player_inventory, max_area_level);

    let player_resources = PlayerResources {
        gold: 1_000_000.0,
        experience: 1_000_000.0,
        ..Default::default()
    };

    let player_base_specs = player_updater::init_player_base_specs(
        "Dictionary Trainer".to_string(),
        "default".to_string(),
        max_area_level,
        Default::default(),
    );
    let player_controller = PlayerController::init(&player_base_specs);

    let mut game_data = GameInstanceData::init_from_store(
        master_store,
        Realm::Standard.realm_id(),
        area_id,
        None,
        max_area_level,
        "default",
        PassivesTreeState::default(),
        player_resources,
        player_base_specs,
        player_inventory,
        Duration::from_secs(600),
        player_controller,
    )?;

    player_controller::init_skills_from_inventory(
        game_data.player_base_specs.mutate(),
        game_data.player_inventory.mutate(),
        &mut game_data.player_state,
        &mut game_data.player_controller,
    );

    for _ in 0..max_area_level.min(20) {
        player_controller::level_up_no_cost(
            game_data.player_base_specs.mutate(),
            &mut game_data.player_state,
            game_data.player_resources.mutate(),
        );
    }

    let mut skill_ids = master_store
        .skills_store
        .keys()
        .cloned()
        .collect::<Vec<_>>();
    skill_ids.sort();
    for skill_id in skill_ids.into_iter().take(3) {
        player_controller::buy_skill(
            &master_store.skills_store,
            game_data.player_base_specs.mutate(),
            &mut game_data.player_state,
            &mut game_data.player_controller,
            game_data.player_resources.mutate(),
            &skill_id,
        );
    }

    Ok(game_data)
}

fn mutate_scenario(master_store: &MasterStore, game_data: &mut GameInstanceData, tick: usize) {
    if tick == 10 {
        game_data.area_state.mutate().rush_mode = true;
    }
    if tick == 60 {
        game_data.area_state.mutate().going_back = 1;
    }
    if tick.is_multiple_of(47) {
        player_controller::level_up(
            game_data.player_base_specs.mutate(),
            &mut game_data.player_state,
            game_data.player_resources.mutate(),
        );
    }
    if tick.is_multiple_of(11) {
        make_all_skills_ready(game_data);
    }
    if tick.is_multiple_of(13) {
        apply_status_samples(game_data, tick);
    }
    if tick.is_multiple_of(71)
        && let Some(item) = loot_generator::generate_loot(
            &game_data.area_blueprint.loot_table,
            &master_store.items_store,
            &master_store.item_affixes_table,
            &master_store.item_adjectives_table,
            &master_store.item_nouns_table,
            game_data.area_state.read().area_level,
            0,
            game_data.area_state.read().is_boss,
            true,
            false,
            false,
            None,
            *game_data.area_specs.loot_rarity,
            game_data.player_specs.read().gold_find.get(),
        )
    {
        let _ = inventory_controller::store_item_to_bag(game_data.player_inventory.mutate(), item);
    }
}

fn equip_sample_item(
    master_store: &MasterStore,
    inventory: &mut PlayerInventory,
    item_id: &str,
    rarity: ItemRarity,
    level: u16,
) {
    let Some(base_item) = master_store.items_store.content.get(item_id).cloned() else {
        return;
    };

    let item = loot_generator::roll_item(
        item_id.to_string(),
        base_item,
        rarity,
        level,
        0,
        &master_store.item_affixes_table,
        &master_store.item_adjectives_table,
        &master_store.item_nouns_table,
        false,
        0.0,
    );
    let _ = inventory_controller::equip_item(inventory, item);
}

fn add_unique_item_samples(
    master_store: &MasterStore,
    inventory: &mut PlayerInventory,
    max_area_level: u16,
) {
    let mut unique_item_ids = master_store
        .items_store
        .content
        .iter()
        .filter_map(|(item_id, item)| (item.rarity == ItemRarity::Unique).then_some(item_id))
        .cloned()
        .collect::<Vec<_>>();
    unique_item_ids.sort();

    for item_id in unique_item_ids.into_iter().take(12) {
        let Some(base_item) = master_store.items_store.content.get(&item_id).cloned() else {
            continue;
        };

        let item = loot_generator::roll_item(
            item_id,
            base_item,
            ItemRarity::Unique,
            max_area_level,
            100,
            &master_store.item_affixes_table,
            &master_store.item_adjectives_table,
            &master_store.item_nouns_table,
            false,
            0.0,
        );
        let _ = inventory_controller::store_item_to_bag(inventory, item);
    }
}

fn make_all_skills_ready(game_data: &mut GameInstanceData) {
    for skill_state in &mut game_data.player_state.character_state.skills_states {
        skill_state.elapsed_cooldown = 1.0.into();
        skill_state.is_ready = true;
    }

    for monster_state in &mut game_data.monster_states {
        for skill_state in &mut monster_state.character_state.skills_states {
            skill_state.elapsed_cooldown = 1.0.into();
            skill_state.is_ready = true;
        }
    }
}

fn apply_status_samples(game_data: &mut GameInstanceData, tick: usize) {
    add_status_samples_to_map(
        &mut game_data.player_state.character_state.statuses,
        SkillType::Blessing,
        tick,
        false,
    );
    game_data.player_state.character_state.dirty_specs = true;

    for monster_state in &mut game_data.monster_states {
        add_status_samples_to_map(
            &mut monster_state.character_state.statuses,
            SkillType::Curse,
            tick,
            true,
        );
        monster_state.character_state.dirty_specs = true;
    }
}

fn add_status_samples_to_map(
    statuses: &mut StatusMap,
    skill_type: SkillType,
    tick: usize,
    hostile: bool,
) {
    let duration = Some((3.0 + (tick % 7) as f64 * 0.25).into());
    let dot_specs = StatusSpecs::DamageOverTime {
        damage_type: if hostile {
            DamageType::Poison
        } else {
            DamageType::Fire
        },
    };
    statuses.cumulative_statuses.push((
        dot_specs.clone(),
        statuses_controller::initialize_status_state(
            skill_type,
            (8.0 + tick as f64 % 5.0).into(),
            duration,
            12.0,
            true,
        ),
    ));

    let damage_specs = StatusSpecs::StatModifier {
        stat: StatType::Damage {
            skill_filter: StatSkillFilter {
                skill_type: Some(if hostile {
                    SkillType::Attack
                } else {
                    SkillType::Spell
                }),
                skill_id: None,
                skill_description: None,
            },
            damage_type: Some(DamageType::Physical),
            min_max: None,
            is_hit: None,
        },
        modifier: Modifier::Increased,
        debuff: hostile,
    };
    statuses.unique_statuses.insert(
        damage_specs.clone().into_status_id(skill_type),
        (
            damage_specs,
            statuses_controller::initialize_status_state(
                skill_type,
                (20.0 + tick as f64 % 13.0).into(),
                Some(5.0.into()),
                0.0,
                false,
            ),
        ),
    );

    let resistance_specs = StatusSpecs::StatModifier {
        stat: StatType::DamageResistance {
            skill_type: Some(SkillType::Attack),
            damage_type: Some(DamageType::Storm),
        },
        modifier: Modifier::Flat,
        debuff: hostile,
    };
    statuses.unique_statuses.insert(
        resistance_specs.clone().into_status_id(skill_type),
        (
            resistance_specs,
            statuses_controller::initialize_status_state(
                skill_type,
                (10.0 + tick as f64 % 9.0).into(),
                Some(4.0.into()),
                4.0,
                false,
            ),
        ),
    );

    if hostile {
        statuses.unique_statuses.insert(
            StatusSpecs::Stun.into_status_id(skill_type),
            (
                StatusSpecs::Stun,
                statuses_controller::initialize_status_state(
                    skill_type,
                    1.0.into(),
                    Some(0.4.into()),
                    0.0,
                    false,
                ),
            ),
        );
    }
}

fn push_sync_sample(samples: &mut Vec<Vec<u8>>, game_data: &mut GameInstanceData) -> Result<()> {
    let message = ServerMessage::from(game_sync::build_sync_update_message(game_data));
    samples.push(rmp_serde::to_vec(&message)?);
    Ok(())
}

fn compressed_len_without_dict(samples: &[Vec<u8>]) -> Result<usize> {
    samples
        .iter()
        .map(|sample| {
            zstd::bulk::compress(sample, 1)
                .map(|compressed| compressed.len())
                .map_err(Into::into)
        })
        .sum()
}

fn compressed_len_with_dict(samples: &[Vec<u8>], dictionary: &[u8]) -> Result<usize> {
    let mut compressor = zstd::bulk::Compressor::with_dictionary(1, dictionary)?;

    samples
        .iter()
        .map(|sample| {
            compressor
                .compress(sample)
                .map(|compressed| compressed.len())
                .map_err(Into::into)
        })
        .sum()
}

fn write_samples(sample_dir: &Path, samples: &[Vec<u8>]) -> Result<()> {
    fs::create_dir_all(sample_dir)
        .with_context(|| format!("failed to create '{}'", sample_dir.display()))?;
    for (index, sample) in samples.iter().enumerate() {
        fs::write(sample_dir.join(format!("{index:05}.msgpack")), sample)?;
    }
    Ok(())
}

struct Config {
    data_dir: PathBuf,
    output: PathBuf,
    sample_dir: Option<PathBuf>,
    ticks_per_scenario: usize,
    dict_size: usize,
    check_only: bool,
}

impl Config {
    fn from_args() -> Result<Self> {
        let mut config = Self {
            data_dir: DEFAULT_DATA_DIR.into(),
            output: DEFAULT_OUTPUT.into(),
            sample_dir: None,
            ticks_per_scenario: DEFAULT_TICKS_PER_SCENARIO,
            dict_size: DEFAULT_DICT_SIZE,
            check_only: false,
        };

        let mut args = env::args().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--data-dir" => config.data_dir = next_path(&mut args, "--data-dir")?,
                "--output" => config.output = next_path(&mut args, "--output")?,
                "--sample-dir" => config.sample_dir = Some(next_path(&mut args, "--sample-dir")?),
                "--default-sample-dir" => config.sample_dir = Some(DEFAULT_SAMPLE_DIR.into()),
                "--ticks" => {
                    config.ticks_per_scenario = next_value(&mut args, "--ticks")?
                        .parse()
                        .context("--ticks must be a number")?
                }
                "--dict-size" => {
                    config.dict_size = next_value(&mut args, "--dict-size")?
                        .parse()
                        .context("--dict-size must be a number")?
                }
                "--check" => config.check_only = true,
                "--help" | "-h" => {
                    print_help();
                    std::process::exit(0);
                }
                _ => anyhow::bail!("unknown argument '{arg}'. Use --help for usage."),
            }
        }

        Ok(config)
    }
}

fn next_value(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<String> {
    args.next()
        .with_context(|| format!("missing value after {flag}"))
}

fn next_path(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<PathBuf> {
    Ok(next_value(args, flag)?.into())
}

fn print_help() {
    println!(
        "Usage: cargo run -p backend --bin train_sync_zstd_dictionary -- [options]\n\
         \n\
         Options:\n\
         --data-dir <path>      Data directory to load. Default: {DEFAULT_DATA_DIR}\n\
         --output <path>        Dictionary output path. Default: {DEFAULT_OUTPUT}\n\
         --ticks <n>            Ticks per area scenario. Default: {DEFAULT_TICKS_PER_SCENARIO}\n\
         --dict-size <bytes>    Max dictionary size. Default: {DEFAULT_DICT_SIZE}\n\
         --sample-dir <path>    Also write generated MessagePack samples\n\
         --default-sample-dir   Write samples to {DEFAULT_SAMPLE_DIR}\n\
         --check                Train and print stats without writing output"
    );
}
