use std::time::{Duration, Instant};

use crate::{db, game::sessions::SessionsStore};

pub async fn purge_sessions(db_pool: db::DbPool, sessions_store: SessionsStore) {
    loop {
        tokio::time::sleep(Duration::from_secs(60)).await;
        let purge_before = Instant::now() - Duration::from_secs(300);

        let mut dropped_sessions = Vec::new();

        {
            let sessions_to_drop: Vec<_> = sessions_store
                .sessions
                .iter()
                .filter_map(|entry| {
                    if entry.value().last_active < purge_before {
                        Some(entry.key().clone())
                    } else {
                        None
                    }
                })
                .collect();

            for character_id in sessions_to_drop {
                if let Some((_, session)) = sessions_store.sessions.remove(&character_id) {
                    dropped_sessions.push((character_id, session));
                }
            }
        }

        for (character_id, session) in dropped_sessions {
            if let Err(e) = db::game_instances::save_game_instance_data(
                &db_pool,
                &session.character_id,
                *session.game_data,
            )
            .await
            {
                tracing::error!(
                    "failed to save game instance from session '{}': {}",
                    character_id,
                    e
                );
            }
        }
    }
}
