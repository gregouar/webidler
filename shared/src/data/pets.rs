use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct PetSpecs {
    pub name: String,
    pub icon: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PetButton {
    Skill1,
    Skill2,
    Skill3,
    Skill4,
    LevelUp,
    AutoPassive,
}

pub type PlayerPets = HashMap<PetButton, String>;

impl PetButton {
    pub fn from_skill_index(index: usize) -> Self {
        use PetButton::*;
        match index {
            0 => Skill1,
            1 => Skill2,
            2 => Skill3,
            3 => Skill4,
            _ => Skill1,
        }
    }

    pub fn to_skill_index(&self) -> Option<usize> {
        match self {
            PetButton::Skill1 => Some(0),
            PetButton::Skill2 => Some(1),
            PetButton::Skill3 => Some(2),
            PetButton::Skill4 => Some(3),
            PetButton::LevelUp => None,
            PetButton::AutoPassive => None,
        }
    }
}
