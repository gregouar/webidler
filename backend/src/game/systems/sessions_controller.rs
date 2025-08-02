use std::{collections::HashMap, time::Instant};

use anyhow::{anyhow, Result};
use rand::TryRngCore;

use shared::{
    data::{
        character::CharacterSize,
        item::ItemRarity,
        player::{CharacterSpecs, PlayerInventory, PlayerResources, PlayerSpecs, PlayerState},
    },
    messages::UserId,
};

use crate::{
    db,
    game::{
        data::{master_store::MasterStore, DataInit},
        game_data::GameInstanceData,
        sessions::{Session, SessionId, SessionKey, SessionsStore},
        systems::leaderboard_controller,
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
    user_id: &UserId,
) -> Result<(SessionId, Session)> {
    tracing::debug!("create new session for player '{user_id}'...");

    if db::game_sessions::is_user_in_session(db_pool, user_id).await? {
        return Err(anyhow!("player already in session"));
    }

    let game_instance_data =
        match load_and_remove_game_instance(db_pool, master_store, user_id).await {
            Some(saved_instance) => saved_instance,
            None => new_game_instance(master_store, user_id)?,
        };

    let session_id = db::game_sessions::create_session(db_pool, user_id).await?;

    let mut rng = rand::rng();
    let mut session_key: SessionKey = [0u8; 32];
    rng.try_fill_bytes(&mut session_key)?;

    Ok((
        session_id,
        Session {
            user_id: user_id.to_string(),
            session_key,
            last_active: Instant::now(),
            game_data: Box::new(game_instance_data),
        },
    ))
}

async fn load_and_remove_game_instance(
    db_pool: &db::DbPool,
    master_store: &MasterStore,
    user_id: &UserId,
) -> Option<GameInstanceData> {
    let saved_game_instance =
        match db::game_instances::load_game_instance_data(db_pool, master_store, user_id).await {
            Ok(x) => x,
            Err(e) => {
                tracing::error!("failed to load game instance for user '{}': {}", user_id, e);
                None
            }
        };

    if let Err(e) = db::game_instances::delete_game_instance_data(db_pool, user_id).await {
        tracing::error!(
            "failed to delete saved game instance for user '{}': {}",
            user_id,
            e
        );
    };

    saved_game_instance
}

fn new_game_instance(master_store: &MasterStore, user_id: &UserId) -> Result<GameInstanceData> {
    let mut player_specs = PlayerSpecs::init(CharacterSpecs {
        name: user_id.to_string(), // TODO: LOL
        portrait: String::from("adventurers/human_male_2.webp"),
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

    let mut player_inventory = PlayerInventory {
        max_bag_size: 40,
        ..Default::default()
    };

    let player_resources = PlayerResources::default();

    let mut player_state = PlayerState::init(&player_specs); // How to avoid this?

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

    GameInstanceData::init_from_store(
        master_store,
        "inn_basement.json",
        None,
        "default",
        None,
        player_resources,
        player_specs,
        player_inventory,
        None,
        None,
    )
}

pub async fn end_session(
    db_pool: &db::DbPool,
    session_id: &SessionId,
    session: &Session,
) -> Result<()> {
    if let Err(e) =
        leaderboard_controller::save_game_score(db_pool, session_id, &session.game_data).await
    {
        tracing::error!("failed to save game score: {}", e);
    }
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
            end_session(&db_pool, &session_id, &session).await?;
            db::game_instances::save_game_instance_data(
                &db_pool,
                &session.user_id,
                *session.game_data,
            )
            .await
        })
    }))
    .await;

    Ok(())
}
