use std::time::{Duration, Instant};

use anyhow::Result;

use shared::data::{
    area::AreaLevel,
    character::CharacterSize,
    item::ItemRarity,
    passive::PassivesTreeState,
    player::{CharacterSpecs, PlayerInventory, PlayerResources, PlayerSpecs},
    temple::PlayerBenedictions,
    user::UserCharacterId,
};

use crate::{
    db::{self, characters::CharacterEntry},
    game::{
        data::{
            inventory_data::inventory_data_to_player_inventory, master_store::MasterStore, DataInit,
        },
        game_data::GameInstanceData,
        sessions::{Session, SessionsStore},
        systems::inventory_controller,
    },
    rest::AppError,
};

use super::{loot_generator, player_controller};

pub async fn create_session(
    db_pool: &db::DbPool,
    sessions_store: &SessionsStore,
    master_store: &MasterStore,
    character: CharacterEntry,
    area_id: &Option<String>,
) -> Result<Session> {
    let character_id = character.character_id;
    tracing::debug!("create new session for player '{character_id}'...");

    let mut first_try = true;
    let mut session_id = None;
    for _ in 0..50 {
        session_id = db::game_sessions::create_session(db_pool, &character_id).await?;
        match session_id {
            Some(_) => break,
            None => {
                if first_try {
                    sessions_store
                        .sessions_stealing
                        .lock()
                        .unwrap()
                        .insert(character_id);
                }
                first_try = false;
                tokio::time::sleep(Duration::from_millis(200)).await;
            }
        }
    }
    if session_id.is_none() {
        return Err(AppError::UserError("character already in session".to_string()).into());
    }

    // First try to get session from memory
    if let Some(session) = {
        let mut sessions_store = sessions_store.sessions.lock().unwrap();
        sessions_store.remove(&character_id)
    } {
        return Ok(session);
    }

    // If not available, try from saved games, otherwise start new game
    let game_instance_data = match load_game_instance(db_pool, master_store, &character_id).await {
        Some(saved_instance) => saved_instance,
        None => {
            let area_id = area_id.as_ref().ok_or(anyhow::anyhow!("missing area id"))?;
            new_game_instance(db_pool, master_store, character, area_id).await?
        }
    };

    Ok(Session {
        character_id,
        last_active: Instant::now(),
        game_data: Box::new(game_instance_data),
    })
}

async fn load_game_instance(
    db_pool: &db::DbPool,
    master_store: &MasterStore,
    character_id: &UserCharacterId,
) -> Option<GameInstanceData> {
    let saved_game_instance = match db::game_instances::load_game_instance_data(
        db_pool,
        master_store,
        character_id,
    )
    .await
    {
        Ok(x) => x,
        Err(e) => {
            tracing::error!(
                "failed to load game instance for character '{}': {}",
                character_id,
                e
            );
            None
        }
    };

    saved_game_instance
}

async fn new_game_instance(
    db_pool: &db::DbPool,
    master_store: &MasterStore,
    character: CharacterEntry,
    area_id: &str,
) -> Result<GameInstanceData> {
    let player_specs = PlayerSpecs::init(CharacterSpecs {
        name: character.character_name.clone(),
        portrait: character.portrait.clone(),
        size: CharacterSize::Small,
        position_x: 0,
        position_y: 0,
        max_life: 100.0,
        life_regen: 10.0,
        max_mana: 100.0,
        mana_regen: 10.0,
        ..Default::default()
    });

    let player_resources = PlayerResources::default();

    let character_data =
        db::characters_data::load_character_data(db_pool, &character.character_id).await?;

    let area_level_completed =
        db::characters::read_character_area_completed(db_pool, &character.character_id, area_id)
            .await?
            .map(|area_completed| area_completed.max_area_level)
            .unwrap_or_default();

    let (player_inventory, passives_tree_state, player_benedictions) = match character_data {
        Some((inventory_data, ascension, player_benedictions)) => (
            inventory_data_to_player_inventory(&master_store.items_store, inventory_data),
            PassivesTreeState::init(ascension),
            player_benedictions,
        ),
        None => {
            let mut player_inventory = PlayerInventory {
                max_bag_size: 40,
                ..Default::default()
            };

            let base_weapon_id = "dagger".to_string();
            if let Some(base_weapon) = master_store.items_store.get(&base_weapon_id).cloned() {
                let _ = inventory_controller::equip_item(
                    &mut player_inventory,
                    loot_generator::roll_item(
                        base_weapon_id,
                        base_weapon,
                        ItemRarity::Normal,
                        0,
                        &master_store.item_affixes_table,
                        &master_store.item_adjectives_table,
                        &master_store.item_nouns_table,
                    ),
                );
            }

            (
                player_inventory,
                PassivesTreeState::default(),
                PlayerBenedictions::default(),
            )
        }
    };

    let mut game_data = GameInstanceData::init_from_store(
        master_store,
        area_id,
        area_level_completed as AreaLevel,
        "default",
        passives_tree_state,
        player_benedictions,
        player_resources,
        player_specs,
        player_inventory,
    )?;

    player_controller::init_skills_from_inventory(
        game_data.player_specs.mutate(),
        game_data.player_inventory.mutate(),
        &mut game_data.player_state,
    );

    if game_data.player_specs.mutate().skills_specs.is_empty() {
        game_data.player_specs.mutate().buy_skill_cost = 0.0;
    }

    // Only the delta is saved in db, so we adjust by starting_level
    game_data.area_state.mutate().max_area_level_ever +=
        game_data.area_blueprint.specs.starting_level - 1;
    game_data.area_state.mutate().last_champion_spawn +=
        game_data.area_blueprint.specs.starting_level - 1;

    game_data.player_resources.mutate().gold += game_data.area_blueprint.specs.starting_gold;

    db::game_instances::save_game_instance_data(
        db_pool,
        &character.character_id,
        game_data.clone(),
    )
    .await?;

    Ok(game_data)
}

pub async fn save_all_sessions(db_pool: &db::DbPool, sessions_store: &SessionsStore) -> Result<()> {
    let sessions = sessions_store
        .sessions
        .lock()
        .unwrap()
        .drain()
        .collect::<Vec<_>>();

    // TODO: Trace errors
    futures::future::join_all(sessions.into_iter().map(|(_, session)| {
        let db_pool = db_pool.clone();
        tokio::spawn(async move {
            db::game_instances::save_game_instance_data(
                &db_pool,
                &session.character_id,
                *session.game_data,
            )
            .await
        })
    }))
    .await;

    Ok(())
}
