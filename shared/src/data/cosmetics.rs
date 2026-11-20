use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum CosmeticType {
    Badge(BadgeSpecs),
    Portrait(PortraitSpecs),
    Title(String),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct PortraitSpecs {
    pub image: String,
    #[serde(default)]
    pub locked: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct BadgeSpecs {
    pub name: String,
    pub description: String,
    pub icon: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct CharacterCosmetics {
    pub title: Option<String>,
    pub badge: Option<String>,
}
