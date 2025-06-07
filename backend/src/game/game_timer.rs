use std::time::{Duration, Instant};

const LOOP_MIN_PERIOD: Duration = Duration::from_millis(100);
const AUTOSAVE_SCORE_DELAY: Duration = Duration::from_secs(60);

#[derive(Debug, Clone)]
pub struct GameTimer {
    last_tick: Instant,
    last_update: Instant,
    last_autosave: Instant,
}

impl GameTimer {
    pub fn new() -> Self {
        Self {
            last_tick: Instant::now(),
            last_update: Instant::now(),
            last_autosave: Instant::now(),
        }
    }

    pub fn delta(&mut self) -> Duration {
        let now = Instant::now();
        let dt = now - self.last_update;
        self.last_update = now;
        dt
    }

    pub async fn wait_tick(&mut self) {
        let now = Instant::now();
        let elapsed = now - self.last_tick;
        if elapsed < LOOP_MIN_PERIOD {
            tokio::time::sleep(LOOP_MIN_PERIOD - elapsed).await;
        }
        self.last_tick = Instant::now();
    }

    pub fn should_autosave(&mut self) -> bool {
        let now = Instant::now();
        if Instant::now() - self.last_autosave > AUTOSAVE_SCORE_DELAY {
            self.last_autosave = now;
            true
        } else {
            false
        }
    }
}
