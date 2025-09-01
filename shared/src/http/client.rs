use serde::{Deserialize, Serialize};

use crate::{
    data::{market::MarketItem, passive::PassivesTreeAscension, user::UserCharacterId},
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
pub struct GetMarketItemsRequest {
    pub character_id: UserCharacterId,
    // TODO filters, order by etc
    // Pagination
    pub skip: usize,
    pub limit: PaginationLimit,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SellMarketItemRequest {
    pub character_id: UserCharacterId,
    pub private_offer: Option<Username>,
    pub market_item: MarketItem,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BuyMarketItemRequest {
    pub character_id: UserCharacterId,
    pub item_index: usize,
}
