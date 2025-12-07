use chrono::{DateTime, Utc};
use std::{collections::HashMap, fmt, time::Duration};

use serde::{Deserialize, Serialize};

use crate::data::{
    area::{AreaLevel, AreaSpecs},
    game_stats::GrindStats,
    market::MarketItem,
    passive::{PassivesTreeAscension, PassivesTreeSpecs},
    player::PlayerInventory,
    skill::SkillSpecs,
    stash::{Stash, StashItem},
    temple::{BenedictionSpecs, PlayerBenedictions},
    user::{User, UserCharacter, UserCharacterId, UserDetails, UserGrindArea, UserId},
};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ErrorResponse {
    pub error: String,
}

impl fmt::Display for ErrorResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.error)
    }
}

// Public

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LeaderboardEntry {
    pub user_id: UserId,
    pub username: String,
    pub character_id: UserCharacterId,
    pub character_name: String,

    pub area_id: String,
    pub area_level: AreaLevel,
    pub created_at: DateTime<Utc>,
    pub elapsed_time: Option<Duration>,
    pub comments: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NewsEntry {
    pub content: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlayersCountResponse {
    pub value: i64,
    pub glimpse: Vec<LeaderboardEntry>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct LeaderboardResponse {
    pub entries: Vec<LeaderboardEntry>,
}
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct NewsResponse {
    pub entries: Vec<NewsEntry>,
}

// Users

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SignUpResponse {}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SignInResponse {
    pub jwt: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ForgotPasswordResponse {}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ResetPasswordResponse {}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GetUserResponse {
    pub user: User,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GetUserDetailsResponse {
    pub user_details: UserDetails,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct UpdateAccountResponse {}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct DeleteAccountResponse {}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GetDiscordInviteResponse {
    pub code: String,
}

// Characters

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CreateCharacterResponse {
    pub character_id: UserCharacterId,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GetUserCharactersResponse {
    pub characters: Vec<UserCharacter>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GetCharacterDetailsResponse {
    pub character: UserCharacter,
    pub areas: Vec<UserGrindArea>,
    pub inventory: PlayerInventory,
    pub ascension: PassivesTreeAscension,
    pub benedictions: PlayerBenedictions,

    pub user_stash: Option<Stash>,
    pub market_stash: Option<Stash>,

    pub last_grind: Option<GrindStats>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct DeleteCharacterResponse {}

// Game

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GetAreasResponse {
    pub areas: HashMap<String, AreaSpecs>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GetSkillsResponse {
    pub skills: HashMap<String, SkillSpecs>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GetPassivesResponse {
    pub passives_tree_specs: PassivesTreeSpecs,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GetBenedictionsResponse {
    pub benedictions_specs: HashMap<String, BenedictionSpecs>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AscendPassivesResponse {
    pub character: UserCharacter,
    pub ascension: PassivesTreeAscension,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct BuyBenedictionsResponse {
    pub character: UserCharacter,
    pub player_benedictions: PlayerBenedictions,
}

// Market

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct BrowseMarketItemsResponse {
    pub items: Vec<MarketItem>,
    pub has_more: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SellMarketItemResponse {
    pub inventory: PlayerInventory,
    pub stash: Stash,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EditMarketItemResponse {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BuyMarketItemResponse {
    pub resource_gems: f64,
    pub inventory: PlayerInventory,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RejectMarketItemResponse {}

// Stash

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct BrowseStashItemsResponse {
    pub items: Vec<StashItem>,
    pub has_more: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StoreStashItemResponse {
    pub inventory: PlayerInventory,
    pub stash: Stash,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TakeStashItemResponse {
    pub inventory: PlayerInventory,
    pub stash: Stash,
}

// Forge

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ForgeAddAffixResponse {
    pub resource_gems: f64,
    pub inventory: PlayerInventory,
}

// Inventory

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InventoryEquipResponse {
    pub inventory: PlayerInventory,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InventoryUnequipResponse {
    pub inventory: PlayerInventory,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InventoryDeleteResponse {
    pub inventory: PlayerInventory,
}
