use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HelloResponse {
    pub greeting: String,
    pub value: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlayersCountResponse {
    pub value: usize,
}
