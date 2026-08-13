use serde::{Deserialize, Serialize};

pub type RealmId = String;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Realm {
    #[default]
    Standard,
    StandardSSF,
    Legacy,
    LegacySSF,
}

impl Realm {
    pub fn realm_id(&self) -> String {
        match self {
            Realm::Standard => "Standard",
            Realm::StandardSSF => "StandardSSF",
            Realm::Legacy => "Legacy",
            Realm::LegacySSF => "LegacySSF",
        }
        .into()
    }

    pub fn allow_parallel_characters(&self) -> bool {
        matches!(self, Realm::LegacySSF)
    }

    pub fn is_ssf(&self) -> bool {
        matches!(self, Realm::StandardSSF | Realm::LegacySSF)
    }
}

impl From<&RealmId> for Realm {
    fn from(value: &RealmId) -> Self {
        match value.as_str() {
            "Legacy" => Realm::Legacy,
            "LegacySSF" => Realm::LegacySSF,
            "StandardSSF" => Realm::StandardSSF,
            _ => Realm::Standard,
        }
    }
}
