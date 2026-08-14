use std::collections::HashSet;

use shared::data::{
    character_status::StatusId,
    conditional_modifier::Condition,
    modifier::{Modifier, compute_more_factor},
    skill::{DamageType, RestoreType, SkillType},
    stat_effect::{EffectsMap, Matchable, StatEffect, StatType, compare_options},
};

/// Computes the multiplier shared by a broad kind of damage.
///
/// The statistics panel deliberately limits a kind to hit/damage over time, damage type,
/// and skill type. More specific effects (a single skill/status or min/max
/// damage) cannot be represented accurately by such a row, so they are left
/// out.
pub fn compute_stats_effects_damage_value(
    effects_map: &EffectsMap,
    skill_type: SkillType,
    damage_type: DamageType,
    is_hit: bool,
) -> f64 {
    let mut factor = Factor::new();

    for effect in effects_map.iter() {
        match &effect.stat {
            StatType::Damage {
                skill_filter,
                damage_type: effect_damage_type,
                min_max,
                is_hit: effect_is_hit,
            } if skill_filter.skill_id.is_none()
                && min_max.is_none()
                && skill_filter.is_match_with_skill(skill_type, &String::new())
                && compare_options(effect_damage_type, &Some(damage_type))
                && compare_options(effect_is_hit, &Some(is_hit)) =>
            {
                factor.apply_effect(&effect);
            }
            StatType::StatusPower {
                status_filter,
                skill_filter,
                min_max,
            } if !is_hit
                && status_filter.status_id.is_none()
                && status_filter.debuff.is_none()
                && skill_filter.skill_id.is_none()
                && min_max.is_none()
                && skill_filter.is_match_with_skill(skill_type, &String::new())
                && status_filter
                    .damage_type
                    .map(|filter| filter.is_match(&damage_type.into()))
                    .unwrap_or(true) =>
            {
                factor.apply_effect(&effect);
            }
            _ => {}
        }
    }

    factor.evaluate()
}

/// Computes the extra critical-damage multiplier for a skill type.
/// Flat critical damage is intentionally excluded from this aggregate.
pub fn compute_stats_effects_crit_damage_value(
    effects_map: &EffectsMap,
    skill_type: SkillType,
) -> f64 {
    let mut factor = Factor::new();
    let skill_id = String::new();

    for effect in effects_map.iter() {
        if let StatType::CritDamage(skill_filter) = &effect.stat
            && skill_filter.skill_id.is_none()
            && skill_filter.is_match_with_skill(skill_type, &skill_id)
        {
            factor.apply_effect(&effect);
        }
    }

    factor.evaluate()
}

pub fn compute_stats_effects_speed_value(effects_map: &EffectsMap, skill_type: SkillType) -> f64 {
    let mut factor = Factor::new();
    let skill_id = String::new();

    for effect in effects_map.iter() {
        if let StatType::Speed(skill_filter) = &effect.stat
            && skill_filter.skill_id.is_none()
            && skill_filter.is_match_with_skill(skill_type, &skill_id)
        {
            factor.apply_effect(&effect);
        }
    }

    factor.evaluate()
}

pub fn compute_stats_effects_crit_chance_value(
    effects_map: &EffectsMap,
    skill_type: SkillType,
) -> f64 {
    let mut factor = Factor::new();
    let skill_id = String::new();

    for effect in effects_map.iter() {
        if let StatType::CritChance(skill_filter) = &effect.stat
            && skill_filter.skill_id.is_none()
            && skill_filter.is_match_with_skill(skill_type, &skill_id)
        {
            factor.apply_effect(&effect);
        }
    }

    factor.evaluate()
}

pub fn compute_stats_effects_threat_damage_value(
    effects_map: &EffectsMap,
    skill_type: SkillType,
) -> f64 {
    let mut factor = Factor::new();
    let skill_id = String::new();

    for effect in effects_map.iter() {
        if let StatType::StatConditionalModifier {
            stat,
            conditions,
            conditions_duration: _,
        } = &effect.stat
            && conditions.len() == 1
            && conditions[0] == Condition::ThreatLevel
            && let StatType::Damage {
                skill_filter,
                damage_type,
                min_max,
                is_hit,
            } = stat.as_ref()
            && skill_filter.skill_id.is_none()
            && skill_filter.is_match_with_skill(skill_type, &skill_id)
            && damage_type.is_none()
            && min_max.is_none()
            && is_hit.is_none()
        {
            factor.apply_effect(&effect);
        }
    }

    factor.evaluate()
}

/// Computes a broad status-effect multiplier for one skill type.
/// Status-specific and min/max-specific modifiers are intentionally excluded.
pub fn compute_stats_effects_status_power_value(
    effects_map: &EffectsMap,
    skill_type: SkillType,
) -> f64 {
    let mut factor = Factor::new();
    let skill_id = String::new();

    for effect in effects_map.iter() {
        if let StatType::StatusPower {
            status_filter,
            skill_filter,
            min_max,
        } = &effect.stat
            && status_filter.status_id.is_none()
            && status_filter.damage_type.is_none()
            && status_filter.debuff.is_none()
            && skill_filter.skill_id.is_none()
            && min_max.is_none()
            && skill_filter.is_match_with_skill(skill_type, &skill_id)
        {
            factor.apply_effect(&effect);
        }
    }

    factor.evaluate()
}

/// Computes a broad status-duration multiplier for one skill type.
pub fn compute_stats_effects_status_duration_value(
    effects_map: &EffectsMap,
    skill_type: SkillType,
) -> f64 {
    let mut factor = Factor::new();
    let skill_id = String::new();

    for effect in effects_map.iter() {
        if let StatType::StatusDuration {
            status_filter,
            skill_filter,
        } = &effect.stat
            && status_filter.status_id.is_none()
            && status_filter.damage_type.is_none()
            && status_filter.debuff.is_none()
            && skill_filter.skill_id.is_none()
            && skill_filter.is_match_with_skill(skill_type, &skill_id)
        {
            factor.apply_effect(&effect);
        }
    }

    factor.evaluate()
}

/// Computes the broad restore multiplier for a resource type.
/// Skill-specific restore modifiers cannot be represented by a single row.
pub fn compute_stats_effects_restore_value(
    effects_map: &EffectsMap,
    restore_type: RestoreType,
) -> f64 {
    let mut factor = Factor::new();

    for effect in effects_map.iter() {
        if let StatType::Restore {
            restore_type: effect_restore_type,
            skill_filter,
        } = &effect.stat
            && skill_filter.skill_type.is_none()
            && skill_filter.skill_id.is_none()
            && compare_options(effect_restore_type, &Some(restore_type))
        {
            factor.apply_effect(&effect);
        }
    }

    factor.evaluate()
}

/// Computes the final mana-cost multiplier for one skill type.
pub fn compute_stats_effects_mana_cost_value(
    effects_map: &EffectsMap,
    skill_type: SkillType,
) -> f64 {
    let mut factor = Factor::new();
    let skill_id = String::new();

    for effect in effects_map.iter() {
        if let StatType::ManaCost { skill_filter } = &effect.stat
            && skill_filter.skill_id.is_none()
            && skill_filter.is_match_with_skill(skill_type, &skill_id)
        {
            factor.apply_effect(&effect);
        }
    }

    factor.evaluate()
}

/// Computes the broad success-chance multiplier for one skill type.
/// Effect-specific success modifiers are excluded from the aggregate.
pub fn compute_stats_effects_success_chance_value(
    effects_map: &EffectsMap,
    skill_type: SkillType,
) -> f64 {
    let mut factor = Factor::new();
    let skill_id = String::new();

    for effect in effects_map.iter() {
        if let StatType::SuccessChance {
            skill_filter,
            effect_type,
        } = &effect.stat
            && effect_type.is_none()
            && skill_filter.skill_id.is_none()
            && skill_filter.is_match_with_skill(skill_type, &skill_id)
        {
            factor.apply_effect(&effect);
        }
    }

    factor.evaluate()
}

fn filter_effects(
    effects_map: &EffectsMap,
    ignore_stat_effects: &HashSet<StatType>,
) -> impl Iterator<Item = StatEffect> {
    effects_map.iter().filter(|stat_effect| {
        !ignore_stat_effects
            .iter()
            .any(|ignored_stat_effect| ignored_stat_effect.is_match(&stat_effect.stat))
    })
}

pub fn compute_stats_effects_status_value(
    effects_map: &EffectsMap,
    ignore_stat_effects: &HashSet<StatType>,
    skill_id: Option<&String>,
    skill_type: Option<SkillType>,
    status_id: &StatusId,
    status_damage_type: Option<DamageType>,
    status_debuff: bool,
) -> f64 {
    let mut factor = Factor::new();

    let default_skill_id = "".to_string();
    let skill_id = skill_id.unwrap_or(&default_skill_id);
    let skill_type = skill_type.unwrap_or(SkillType::Other);

    for effect in filter_effects(effects_map, ignore_stat_effects) {
        if let StatType::StatusPower {
            status_filter,
            skill_filter,
            min_max: _,
        } = &effect.stat
            && status_filter.is_match_with_status(status_id, status_damage_type, status_debuff)
            && skill_filter.is_match_with_skill(skill_type, skill_id)
        {
            factor.apply_effect(&effect);
        }

        if let StatType::Damage {
            skill_filter,
            damage_type,
            min_max: _,
            is_hit,
        } = &effect.stat
            && compare_options(is_hit, &Some(false))
            && status_damage_type.is_some()
            && compare_options(&status_damage_type, damage_type)
            && skill_filter.is_match_with_skill(skill_type, skill_id)
        {
            factor.apply_effect(&effect);
        }
    }

    factor.evaluate()
}

pub fn compute_stats_effects_status_duration(
    effects_map: &EffectsMap,
    ignore_stat_effects: &HashSet<StatType>,
    skill_id: Option<&String>,
    skill_type: Option<SkillType>,
    status_id: &StatusId,
    status_damage_type: Option<DamageType>,
    status_debuff: bool,
) -> f64 {
    let mut factor = Factor::new();

    let default_skill_id = "".to_string();
    let skill_id = skill_id.unwrap_or(&default_skill_id);
    let skill_type = skill_type.unwrap_or(SkillType::Other);

    for effect in filter_effects(effects_map, ignore_stat_effects) {
        if let StatType::StatusDuration {
            status_filter,
            skill_filter,
        } = &effect.stat
            && status_filter.is_match_with_status(status_id, status_damage_type, status_debuff)
            && skill_filter.is_match_with_skill(skill_type, skill_id)
        {
            factor.apply_effect(&effect);
        }
    }

    factor.evaluate()
}

struct Factor {
    more: f64,
    increased: f64,
    decreased: f64,
}

impl Factor {
    fn new() -> Self {
        Self {
            more: 0.0,
            increased: 0.0,
            decreased: 0.0,
        }
    }

    fn evaluate(self) -> f64 {
        let div = (1.0 - self.decreased * 0.01).max(0.0);
        // let base = if convert {
        //     self.base.multiply_value(1.0 - self.converted * 0.01)
        // } else {
        //     self.base
        // };

        // if base.is_negative() {
        //     return base;
        // }

        // if self.more == -100.0 {
        //     return base.multiply_value(0.0);
        // }

        // base.multiply_value(factor)

        (1.0 + self.more * 0.01)
            * (1.0 + self.increased * 0.01)
            * (if div > 0.0 { 1.0 / div } else { 1.0 })
    }

    fn apply_effect(&mut self, stat_effect: &StatEffect) {
        match stat_effect.modifier {
            Modifier::Increased => {
                if stat_effect.value >= 0.0 {
                    self.increased += stat_effect.value;
                } else {
                    self.decreased += stat_effect.value;
                }
            }
            Modifier::Flat => {}
            Modifier::More => {
                let value = compute_more_factor(stat_effect.value);
                if value == -100.0 || self.more == -100.0 {
                    self.more = -100.0
                } else {
                    self.more = self.more + value + self.more * value * 0.01;
                }
            }
        }
    }
}
