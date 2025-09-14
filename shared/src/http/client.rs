use serde::{Deserialize, Serialize};

use crate::{
    data::{
        item_affix::AffixType, market::MarketFilters, passive::PassivesTreeAscension,
        user::UserCharacterId,
    },
    types::{AssetName, Email, PaginationLimit, Password, Username},
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SignUpRequest {
    pub captcha_token: String,

    pub username: Username,
    pub email: Option<Email>,
    pub password: Password,
    pub accepted_terms: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SignInRequest {
    pub captcha_token: String,

    pub username: Username,
    pub password: Password,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateCharacterRequest {
    pub name: Username,
    pub portrait: AssetName,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AscendPassivesRequest {
    pub character_id: UserCharacterId,
    pub passives_tree_ascension: PassivesTreeAscension,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct BrowseMarketItemsRequest {
    pub character_id: UserCharacterId,
    pub own_listings: bool,

    pub filters: MarketFilters,

    pub skip: u32,
    pub limit: PaginationLimit,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SellMarketItemRequest {
    pub character_id: UserCharacterId,
    pub recipient_name: Option<Username>,
    pub item_index: usize,
    pub price: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EditMarketItemRequest {
    pub character_id: UserCharacterId,
    pub item_index: u32,
    pub price: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BuyMarketItemRequest {
    pub character_id: UserCharacterId,
    pub item_index: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RejectMarketItemRequest {
    pub character_id: UserCharacterId,
    pub item_index: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ForgeItemRequest {
    pub character_id: UserCharacterId,
    pub item_index: u32,
    pub affix_type: Option<AffixType>,
}
