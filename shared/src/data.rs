// TODO: split in multiple files

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct HelloSchema {
    pub greeting: String,
    pub value: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OtherSchema {
    pub other: String,
    pub value: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CharacterPrototype {
    pub identifier: String,

    pub name: String,
    pub portrait: String,

    pub max_health: f64, // TODO: change to big numbers num_bigint
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CharacterState {
    pub identifier: String, // useful?
    pub health: f64,        // TODO: change to big numbers num_bigint
}
