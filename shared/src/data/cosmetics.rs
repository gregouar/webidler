use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum CosmeticType {
    Badge(BadgeSpecs),
    Pet(PetSpecs),
    Portrait(PortraitSpecs),
    Title(String),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PortraitSpecs {
    pub image: String,
    #[serde(default)]
    pub locked: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PetSpecs {
    pub name: String,
    pub icon: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BadgeSpecs {
    pub name: String,
    pub description: String,
    pub icon: String,
}
