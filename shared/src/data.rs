// TODO: split in multiple files

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct HelloSchema {
    pub greeting: String,
    pub value: i32,
}
