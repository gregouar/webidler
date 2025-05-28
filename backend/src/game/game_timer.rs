use std::time::{Duration, Instant};

const LOOP_MIN_PERIOD: Duration = Duration::from_millis(100);

pub struct GameTimer {
    last_tick: Instant,
    last_update: Instant,
}

impl GameTimer {
    pub fn new() -> Self {
        Self {
            last_tick: Instant::now(),
            last_update: Instant::now(),
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
}
