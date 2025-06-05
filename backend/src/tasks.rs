use std::time::{Duration, Instant};

use crate::{
    db,
    game::session::{Session, SessionsStore},
};

pub async fn purge_sessions(db_pool: db::DbPool, sessions_store: SessionsStore) {
    loop {
        tokio::time::sleep(Duration::from_secs(60)).await;
        let purge_before = Instant::now() - Duration::from_secs(300);

        let mut dropped_sessions = Vec::new();

        {
            let mut sessions = sessions_store.sessions.lock().unwrap();
            let sessions_to_drop: Vec<_> = sessions
                .iter()
                .filter_map(|(k, session)| {
                    if session.last_active < purge_before {
                        Some(k)
                    } else {
                        None
                    }
                })
                .cloned()
                .collect();

            for k in sessions_to_drop {
                if let Some(session) = sessions.remove(&k) {
                    dropped_sessions.push(session);
                }
            }
        }

        for session in dropped_sessions {
            save_session_score(&db_pool, &session).await;
        }
    }
}

pub async fn save_session_score(db_pool: &db::DbPool, session: &Session) {
    if let Err(e) = db::leaderboard::insert_leaderboard_entry(
        &db_pool,
        &session.data.player_specs.read().character_specs.name,
        session.data.game_stats.highest_area_level,
        session.data.game_stats.elapsed_time,
        "",
    )
    .await
    {
        tracing::error!("Failed to save session score to db: {}", e);
    }
}
