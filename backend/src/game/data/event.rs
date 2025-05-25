use std::collections::HashMap;

use shared::data::{character::CharacterId, skill::DamageType};

pub enum GameEvent {
    Hit(DamageEvent),
    CriticalStrike(DamageEvent),
    Block(DamageEvent),
    Kill { target: CharacterId },
}

pub struct DamageEvent {
    attacker: CharacterId,
    target: CharacterId,
    skill: usize,
    damage: HashMap<DamageType, f64>,
}

pub struct EventsQueue {
    events: Vec<GameEvent>,
}

impl EventsQueue {
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    pub fn register_event(&mut self, event: GameEvent) {
        self.events.push(event)
    }

    pub fn consume_events(&mut self) -> Vec<GameEvent> {
        std::mem::take(&mut self.events)
    }
}
