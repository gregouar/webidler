// TODO: Rework to have something data driven

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum UserBadge {
    Developer,
    WitchHunter,
}
