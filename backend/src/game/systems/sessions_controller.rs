use std::{collections::HashMap, time::Instant};

use anyhow::{anyhow, Result};
use rand::TryRngCore;

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
        sessions::{Session, SessionId, SessionKey, SessionsStore},
    },
};

use super::{loot_generator, player_controller};

pub async fn resume_session(
    sessions_store: &SessionsStore,
    session_id: SessionId,
    session_key: SessionKey,
) -> Result<(SessionId, Session)> {
    tracing::debug!("loading player session '{session_id}'...");

    let mut sessions_store = sessions_store.sessions.lock().unwrap();

    match sessions_store.get(&session_id) {
        Some(session) if session.session_key == session_key => {
            Ok((session_id, sessions_store.remove(&session_id).unwrap()))
        }
        _ => Err(anyhow!("couldn't load player session")),
    }
}

pub async fn create_session(
    db_pool: &db::DbPool,
    master_store: &MasterStore,
    character: CharacterEntry,
    area_id: &Option<String>,
) -> Result<(SessionId, Session)> {
    tracing::debug!(
        "create new session for player '{}'...",
        character.character_id
    );

    if db::game_sessions::is_character_id_in_session(db_pool, &character.character_id).await? {
        return Err(anyhow!("character already in session"));
    }

    let game_instance_data =
        match load_game_instance(db_pool, master_store, &character.character_id).await {
            Some(saved_instance) => saved_instance,
            None => {
                let area_id = area_id.as_ref().ok_or(anyhow::anyhow!("missing area id"))?;
                new_game_instance(db_pool, master_store, &character, area_id).await?
            }
        };

    let session_id = db::game_sessions::create_session(db_pool, &character.character_id).await?;

    let mut rng = rand::rng();
    let mut session_key: SessionKey = [0u8; 32];
    rng.try_fill_bytes(&mut session_key)?;

    Ok((
        session_id,
        Session {
            character_id: character.character_id.clone(),
            session_key,
            last_active: Instant::now(),
            game_data: Box::new(game_instance_data),
        },
    ))
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

    // if let Err(e) = db::game_instances::delete_game_instance_data(db_pool, character_id).await {
    //     tracing::error!(
    //         "failed to delete saved game instance for character '{}': {}",
    //         character_id,
    //         e
    //     );
    // };

    saved_game_instance
}

async fn new_game_instance(
    db_pool: &db::DbPool,
    master_store: &MasterStore,
    character: &CharacterEntry,
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
        Some(inventory) => inventory,
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

pub async fn end_session(db_pool: &db::DbPool, session_id: &SessionId) -> Result<()> {
    db::game_sessions::end_session(db_pool, session_id).await?;
    Ok(())
}

pub async fn save_all_sessions(db_pool: &db::DbPool, sessions_store: &SessionsStore) -> Result<()> {
    let sessions = sessions_store
        .sessions
        .lock()
        .unwrap()
        .drain()
        .collect::<Vec<_>>();

    // TODO: Trace errors
    futures::future::join_all(sessions.into_iter().map(|(session_id, session)| {
        let db_pool = db_pool.clone();
        tokio::spawn(async move {
            end_session(&db_pool, &session_id).await?;
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
