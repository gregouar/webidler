use serde::{Deserialize, Serialize};

pub type RealmId = String;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Realm {
    #[default]
    Standard,
    StandardSSF,
    Legacy,
}

impl Realm {
    pub fn realm_id(&self) -> String {
        match self {
            Realm::Standard => "Standard",
            Realm::StandardSSF => "StandardSSF",
            Realm::Legacy => "Legacy",
        }
        .into()
    }
}

impl From<&RealmId> for Realm {
    fn from(value: &RealmId) -> Self {
        match value.as_str() {
            "Legacy" => Realm::Legacy,
            "StandardSSF" => Realm::StandardSSF,
            _ => Realm::Standard,
        }
    }
}
