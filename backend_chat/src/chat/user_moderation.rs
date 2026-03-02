use std::time::Instant;

pub struct UserModerationState {
    tokens: u32,
    last_refill: Instant,
    mute_until: Option<Instant>,
}

impl Default for UserModerationState {
    fn default() -> Self {
        Self::new()
    }
}

impl UserModerationState {
    fn new() -> Self {
        Self {
            tokens: 5,
            last_refill: Instant::now(),
            mute_until: None,
        }
    }

    pub fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs();
        if elapsed > 0 {
            self.tokens = (self.tokens + elapsed as u32).min(5);
            self.last_refill = now;
        }
    }

    pub fn allow_message(&mut self) -> bool {
        self.refill();
        if self.tokens == 0 {
            return false;
        }
        self.tokens -= 1;
        true
    }

    pub fn is_muted(&self) -> bool {
        match self.mute_until {
            Some(t) => Instant::now() < t,
            None => false,
        }
    }
}
