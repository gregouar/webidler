use std::time::{Duration, Instant};

use crate::{db, game::sessions::SessionsStore, game::systems::sessions_controller};

pub async fn purge_sessions(db_pool: db::DbPool, sessions_store: SessionsStore) {
    loop {
        tokio::time::sleep(Duration::from_secs(60)).await;
        let purge_before = Instant::now() - Duration::from_secs(300);

        let mut dropped_sessions = Vec::new();

        {
            let mut sessions = sessions_store.sessions.lock().unwrap();
            let sessions_to_drop: Vec<_> = sessions
                .iter()
                .filter_map(|(session_id, session)| {
                    if session.last_active < purge_before {
                        Some(session_id)
                    } else {
                        None
                    }
                })
                .cloned()
                .collect();

            for session_id in sessions_to_drop {
                if let Some(session) = sessions.remove(&session_id) {
                    dropped_sessions.push((session_id, session));
                }
            }
        }

        for (session_id, session) in dropped_sessions {
            if let Err(e) = sessions_controller::end_session(&db_pool, &session_id, &session).await
            {
                tracing::error!("failed to end game session '{}': {}", session_id, e);
            }
        }
    }
}
