use std::collections::HashMap;

use shared::data::{
    area::{AreaLevel, ThreatLevel},
    character::CharacterId,
    skill::{DamageType, SkillRange, SkillType},
};

#[derive(Debug, Clone)]
pub enum GameEvent {
    Hit(HitEvent),
    Kill { target: CharacterId },
    AreaCompleted(AreaLevel),
    WaveCompleted(AreaLevel),
    ThreatIncreased(ThreatLevel),
}

#[derive(Debug, Clone)]
pub struct HitEvent {
    pub source: CharacterId,
    pub target: CharacterId,

    pub skill_type: SkillType,
    pub range: SkillRange,
    pub is_crit: bool,
    pub is_blocked: bool,
    pub is_hurt: bool,

    pub damage: HashMap<DamageType, f64>,
}

#[derive(Debug, Default)]
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
