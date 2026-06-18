use serde::{Deserialize, Serialize};

use crate::data::item::ItemSpecs;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct QuestRewards {
    pub item_rewards: Vec<ItemSpecs>,
    #[serde(default)]
    pub skill_mastery_rewards: Vec<SkillMasteryReward>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SkillMasteryReward {
    pub skill_id: String,
    pub previous_level: u16,
    pub current_level: u16,
    pub experience_gained: f64,
    pub current_relative_experience: f64,
    pub current_next_level_cost: f64,
}
