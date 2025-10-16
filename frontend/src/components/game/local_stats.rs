use leptos::prelude::*;

use crate::utils;

pub const DAMAGE_TICKS_WINDOWS: f64 = 10.0;

#[derive(Clone, Copy)]
pub struct GameLocalStats {
    damage_ticks: RwSignal<Vec<DamageTick>>,
}

impl GameLocalStats {
    pub fn new() -> Self {
        Self {
            damage_ticks: RwSignal::new(Vec::new()),
        }
    }

    pub fn add_damage_tick(&self, amount: f64) {
        let now = utils::now() * 0.001;
        self.damage_ticks.update(|damage_ticks| {
            damage_ticks.push(DamageTick { amount, when: now });
            damage_ticks.retain(|tick| now - tick.when <= DAMAGE_TICKS_WINDOWS);
        });
    }

    pub fn average_damage_tick(&self) -> f64 {
        self.damage_ticks.with(|damage_ticks| {
            if damage_ticks.is_empty() {
                0.0
            } else {
                damage_ticks.iter().map(|tick| tick.amount).sum::<f64>()
                    / (damage_ticks.len() as f64)
            }
        })
    }

    pub fn average_dps(&self) -> f64 {
        self.damage_ticks.with(|damage_ticks| {
            let total = damage_ticks.iter().map(|tick| tick.amount).sum();
            if let Some((first, last)) = damage_ticks.first().zip(damage_ticks.last()) {
                if first.when != last.when {
                    total / (last.when - first.when)
                } else {
                    total
                }
            } else {
                0.0
            }
        })
    }
}

impl Default for GameLocalStats {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
struct DamageTick {
    pub amount: f64,
    pub when: f64,
}
