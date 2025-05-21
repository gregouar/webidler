use std::time::{Duration, Instant};

use crate::game::session::SessionsStore;

pub async fn purge_sessions(sessions_store: SessionsStore) {
    loop {
        tokio::time::sleep(Duration::from_secs(60)).await;
        let purge_before = Instant::now() - Duration::from_secs(300);

        sessions_store
            .sessions
            .lock()
            .unwrap()
            .retain(|_, session| session.last_active >= purge_before);
    }
}
