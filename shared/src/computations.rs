use crate::{
    constants::{
        self, CHAMPION_BASE_CHANCE, CHAMPION_INC_CHANCE, SKILL_COST_INCREASE_FACTOR,
        SKILL_MASTERY_BASE_COST, SKILL_MASTERY_INCREASE_COST, XP_INCREASE_FACTOR,
    },
    data::{
        area::{AreaLevel, AreaState},
        item::ItemSpecs,
        player::{PlayerBaseSkill, PlayerBaseSpecs},
        stash::{Stash, StashType},
    },
};

pub fn exponential(level: AreaLevel, factor: f64) -> f64 {
    10f64.powf(level.saturating_sub(1) as f64 * factor)
}

// for armor physical damage decrease
pub fn diminishing(amount: f64, factor: f64) -> f64 {
    if amount < 0.0 {
        return 0.0;
    }
    amount / (amount + factor)
}

pub fn skill_cost_increase(player_base_skill: &PlayerBaseSkill) -> f64 {
    player_base_skill.next_upgrade_cost
        + (10.0 * exponential(player_base_skill.upgrade_level, SKILL_COST_INCREASE_FACTOR)).round()
}

pub fn player_level_up_cost(player_specs: &PlayerBaseSpecs) -> f64 {
    (20.0 * exponential(player_specs.level as AreaLevel, XP_INCREASE_FACTOR)).round()
}

pub fn skill_mastery_next_level_cost(level: u16) -> f64 {
    if level == u16::MAX {
        return f64::INFINITY;
    }

    skill_mastery_level_cost(level.saturating_add(1)) - skill_mastery_level_cost(level)
}

pub fn skill_mastery_level_cost(level: u16) -> f64 {
    if level == 0 {
        return 0.0;
    }

    let factor = 10f64.powf(SKILL_MASTERY_INCREASE_COST);
    let level_cost = SKILL_MASTERY_BASE_COST
        * (1.0 + (factor.powi(level.saturating_sub(1) as i32) - 1.0) / (factor - 1.0));

    level_cost.round()
}

pub fn skill_mastery_level(experience: f64) -> u16 {
    if experience < skill_mastery_next_level_cost(0) {
        return 0;
    }

    if !experience.is_finite() {
        return u16::MAX;
    }

    let factor = 10f64.powf(SKILL_MASTERY_INCREASE_COST);
    let inverse_series =
        1.0 + ((experience + 0.5) / SKILL_MASTERY_BASE_COST - 1.0) * (factor - 1.0);
    let estimated_level =
        (inverse_series.log(factor).floor() + 1.0).clamp(0.0, u16::MAX as f64) as u16;

    if skill_mastery_level_cost(estimated_level) > experience {
        estimated_level.saturating_sub(1)
    } else if estimated_level < u16::MAX
        && skill_mastery_level_cost(estimated_level.saturating_add(1)) <= experience
    {
        estimated_level.saturating_add(1)
    } else {
        estimated_level
    }
}

pub fn gem_chance(area_state: &AreaState) -> f64 {
    if area_state.area_level > area_state.last_champion_spawn {
        CHAMPION_BASE_CHANCE
            + (CHAMPION_INC_CHANCE
                * (area_state
                    .area_level
                    .saturating_sub(area_state.last_champion_spawn)) as f64)
    } else {
        0.0
    }
}

pub fn stash_upgrade(stash: &Stash) -> (usize, f64) {
    let stash_price = match stash.stash_type {
        StashType::User => constants::STASH_USER_PRICE,
        StashType::Market => constants::STASH_MARKET_PRICE,
        StashType::Character => constants::STASH_USER_PRICE,
    };

    if stash.max_items < stash_price.start_size {
        return (stash_price.start_size, stash_price.start_price);
    }

    let next_stash_level = stash
        .max_items
        .saturating_sub(stash_price.start_size)
        .div_euclid(stash_price.upgrade_size)
        .saturating_add(1);

    let next_stash_price =
        stash_price.start_price * stash_price.upgrade_price.powi(next_stash_level as i32);

    (
        stash_price.start_size + stash_price.upgrade_size.saturating_mul(next_stash_level),
        next_stash_price,
    )
}

pub fn gamble_price(item_level: AreaLevel) -> f64 {
    (item_level as f64 / 20.0).floor() + 10.0
}

pub fn upgrade_item_price(item_specs: &ItemSpecs) -> Option<f64> {
    item_specs
        .base
        .upgrade_levels
        .get(item_specs.modifiers.upgrade_level as usize)
        .and_then(|next_upgrade_level| {
            (*next_upgrade_level <= item_specs.modifiers.level)
                .then_some((*next_upgrade_level / 2 + item_specs.base.min_area_level) as f64)
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skill_mastery_costs_are_consistent_at_level_boundaries() {
        for level in 1..200 {
            let level_cost = skill_mastery_level_cost(level);

            assert_eq!(skill_mastery_level(level_cost - 1.0), level - 1);
            assert_eq!(skill_mastery_level(level_cost), level);
            assert_eq!(
                skill_mastery_next_level_cost(level - 1),
                level_cost - skill_mastery_level_cost(level - 1)
            );
        }
    }

    #[test]
    fn skill_mastery_level_zero_has_no_accumulated_cost() {
        assert_eq!(skill_mastery_level_cost(0), 0.0);
        assert_eq!(skill_mastery_level(0.0), 0);
        assert_eq!(
            skill_mastery_next_level_cost(0),
            skill_mastery_level_cost(1)
        );
    }
}
