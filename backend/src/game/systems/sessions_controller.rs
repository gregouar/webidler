use std::{collections::HashMap, time::Instant};

use anyhow::Result;

use shared::data::{
    character::CharacterSize,
    item::ItemRarity,
    player::{CharacterSpecs, PlayerInventory, PlayerResources, PlayerSpecs, PlayerState},
    user::UserCharacterId,
};

use crate::{
    db::{self, characters::CharacterEntry},
    game::{
        data::{master_store::MasterStore, DataInit},
        game_data::GameInstanceData,
        sessions::{Session, SessionsStore},
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
    let character_id = character.character_id.clone();
    tracing::debug!("create new session for player '{character_id}'...");

    if let None = db::game_sessions::create_session(db_pool, &character_id).await? {
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
    let mut player_specs = PlayerSpecs::init(CharacterSpecs {
        name: character.character_name.clone(),
        portrait: character.portrait.clone(),
        size: CharacterSize::Small,
        position_x: 0,
        position_y: 0,
        max_life: 100.0,
        life_regen: 1.0,
        max_mana: 100.0,
        mana_regen: 1.0,
        armor: 0.0,
        fire_armor: 0.0,
        poison_armor: 0.0,
        block: 0.0,
        take_from_mana_before_life: 0.0,
        damage_resistance: HashMap::new(),
    });

    let player_resources = PlayerResources {
        experience: 0.0,
        passive_points: 0,
        gold: 0.0,
        gems: character.resource_gems,
        shards: character.resource_shards,
    };

    let character_data =
        db::characters_data::load_character_data(db_pool, &character.character_id).await?;

    let mut player_state = PlayerState::init(&player_specs); // How to avoid this?

    let player_inventory = match character_data {
        Some(inventory) => {
            player_controller::init_item_skills(&mut player_specs, &inventory, &mut player_state);
            inventory
        }
        None => {
            let mut player_inventory = PlayerInventory {
                max_bag_size: 40,
                ..Default::default()
            };

            if let Some(base_weapon) = master_store.items_store.get("dagger").cloned() {
                let _ = player_controller::equip_item(
                    &mut player_specs,
                    &mut player_inventory,
                    &mut player_state,
                    loot_generator::roll_item(
                        base_weapon,
                        ItemRarity::Normal,
                        1,
                        &master_store.item_affixes_table,
                        &master_store.item_adjectives_table,
                        &master_store.item_nouns_table,
                    ),
                );
            }

            player_inventory
        }
    };

    let game_data = GameInstanceData::init_from_store(
        master_store,
        area_id,
        None,
        "default",
        None,
        player_resources,
        player_specs,
        player_inventory,
        None,
        None,
    )?;

    db::game_instances::save_game_instance_data(
        &db_pool,
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
