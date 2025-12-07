use serde::{Deserialize, Serialize};

use crate::{
    data::{
        item::ItemSlot,
        item_affix::AffixType,
        market::MarketFilters,
        passive::PassivesTreeAscension,
        stash::StashType,
        temple::PlayerBenedictions,
        user::{UserCharacterId, UserId},
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
pub struct ForgotPasswordRequest {
    pub captcha_token: String,

    pub email: Email,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResetPasswordRequest {
    pub captcha_token: String,

    pub user_id: UserId,
    pub password: Password,
    pub password_token: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct UpdateAccountRequest {
    pub username: Option<Username>,
    pub email: Option<Option<Email>>,

    pub old_password: Option<Password>,
    pub password: Option<Password>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateCharacterRequest {
    pub name: Username,
    pub portrait: AssetName,
}

// Temple

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct BuyBenedictionsRequest {
    pub character_id: UserCharacterId,
    pub player_benedictions: PlayerBenedictions,
}

// Ascend

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AscendPassivesRequest {
    pub character_id: UserCharacterId,
    pub passives_tree_ascension: PassivesTreeAscension,
}

// Market

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct BrowseMarketItemsRequest {
    pub user_id: UserId,

    pub own_listings: bool,
    pub is_deleted: bool,

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
    pub item_index: u32,
}

// Stash

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct UpgradeStashRequest {
    pub character_id: UserCharacterId,
    pub stash_type: StashType,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct BrowseStashItemsRequest {
    pub character_id: UserCharacterId,

    pub filters: MarketFilters,

    pub skip: u32,
    pub limit: PaginationLimit,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StoreStashItemRequest {
    pub character_id: UserCharacterId,
    pub item_index: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TakeStashItemRequest {
    pub character_id: UserCharacterId,
    pub item_index: u32,
}

// Forge

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ForgeAddAffixRequest {
    pub character_id: UserCharacterId,
    pub item_index: u32,
    pub affix_type: Option<AffixType>,
}

// Inventory

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InventoryEquipRequest {
    pub character_id: UserCharacterId,
    pub item_index: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InventoryUnequipRequest {
    pub character_id: UserCharacterId,
    pub item_slot: ItemSlot,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InventoryDeleteRequest {
    pub character_id: UserCharacterId,
    pub item_indexes: Vec<u8>,
}
