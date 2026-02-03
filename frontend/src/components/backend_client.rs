use reqwest::StatusCode;
use std::time::Duration;

use shared::{
    data::{
        stash::StashId,
        user::{UserCharacterId, UserId},
    },
    http::{
        client::{
            AscendPassivesRequest, BrowseMarketItemsRequest, BrowseStashItemsRequest,
            BuyBenedictionsRequest, BuyMarketItemRequest, CreateCharacterRequest,
            EditMarketItemRequest, ExchangeGemsStashRequest, ForgeAddAffixRequest,
            ForgotPasswordRequest, InventoryDeleteRequest, InventoryEquipRequest,
            InventoryUnequipRequest, RejectMarketItemRequest, ResetPasswordRequest,
            SellMarketItemRequest, SignInRequest, SignUpRequest, SocketPassiveRequest,
            StoreStashItemRequest, TakeStashItemRequest, UpdateAccountRequest,
            UpdateCharacterRequest, UpgradeStashRequest,
        },
        server::{
            AscendPassivesResponse, BrowseMarketItemsResponse, BrowseStashItemsResponse,
            BuyBenedictionsResponse, BuyMarketItemResponse, CreateCharacterResponse,
            DeleteAccountResponse, DeleteCharacterResponse, EditMarketItemResponse, ErrorResponse,
            ExchangeGemsStashResponse, ForgeAddAffixResponse, ForgotPasswordResponse,
            GetAreasResponse, GetBenedictionsResponse, GetCharacterDetailsResponse,
            GetDiscordInviteResponse, GetPassivesResponse, GetSkillsResponse,
            GetUserCharactersResponse, GetUserDetailsResponse, InventoryDeleteResponse,
            InventoryEquipResponse, InventoryUnequipResponse, LeaderboardResponse, NewsResponse,
            PlayersCountResponse, RejectMarketItemResponse, ResetPasswordResponse,
            SellMarketItemResponse, SignInResponse, SignUpResponse, SocketPassiveResponse,
            StoreStashItemResponse, TakeStashItemResponse, UpdateAccountResponse,
            UpgradeStashResponse,
        },
    },
};

#[derive(Clone)]
pub enum BackendError {
    NotFound,
    Unauthorized(String),
    Forbidden,
    UserError(String),
    ServerError(String),
    ServerNotResponding,
    OtherError,
}

impl std::fmt::Display for BackendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BackendError::NotFound => write!(f, "Not found"),
            BackendError::Unauthorized(reason) => write!(f, "Unauthorized: {reason}"),
            BackendError::Forbidden => write!(f, "Forbidden"),
            BackendError::UserError(reason) => write!(f, "{reason}"),
            BackendError::ServerError(reason) => write!(f, "Server error: {reason}"),
            BackendError::ServerNotResponding => write!(f, "Server not responding"),
            BackendError::OtherError => write!(f, "Unknown error"),
        }
    }
}

#[derive(Clone, Copy)]
pub struct BackendClient {
    http_url: &'static str,
    ws_url: &'static str,
}

impl BackendClient {
    pub fn new(http_url: &'static str, ws_url: &'static str) -> Self {
        BackendClient {
            http_url: http_url.trim_end_matches('/'),
            ws_url: ws_url.trim_end_matches('/'),
        }
    }

    // Game

    pub fn get_game_ws_url(&self) -> String {
        format!("{}/ws", self.ws_url)
    }

    pub async fn get_players_count(&self) -> Result<PlayersCountResponse, BackendError> {
        self.get("players").await
    }

    pub async fn get_leaderboard(&self) -> Result<LeaderboardResponse, BackendError> {
        self.get("leaderboard").await
    }

    pub async fn get_news(&self) -> Result<NewsResponse, BackendError> {
        self.get("news").await
    }

    pub async fn get_areas(&self) -> Result<GetAreasResponse, BackendError> {
        self.get("game/areas").await
    }

    pub async fn get_skills(&self) -> Result<GetSkillsResponse, BackendError> {
        self.get("game/skills").await
    }

    pub async fn get_passives(&self) -> Result<GetPassivesResponse, BackendError> {
        self.get("game/passives").await
    }

    pub async fn post_ascend_passives(
        &self,
        token: &str,
        request: &AscendPassivesRequest,
    ) -> Result<AscendPassivesResponse, BackendError> {
        self.post_auth("game/passives", token, request).await
    }

    pub async fn post_socket_passive(
        &self,
        token: &str,
        request: &SocketPassiveRequest,
    ) -> Result<SocketPassiveResponse, BackendError> {
        self.post_auth("game/passives/socket", token, request).await
    }

    pub async fn get_benedictions(&self) -> Result<GetBenedictionsResponse, BackendError> {
        self.get("game/benedictions").await
    }

    pub async fn post_buy_benedictions(
        &self,
        token: &str,
        request: &BuyBenedictionsRequest,
    ) -> Result<BuyBenedictionsResponse, BackendError> {
        self.post_auth("game/benedictions", token, request).await
    }

    // Auth

    pub async fn get_me(&self, token: &str) -> Result<GetUserDetailsResponse, BackendError> {
        self.get_auth("account/me", token).await
    }

    pub async fn post_signin(
        &self,
        request: &SignInRequest,
    ) -> Result<SignInResponse, BackendError> {
        self.post("account/signin", request).await
    }

    pub async fn post_signup(
        &self,
        request: &SignUpRequest,
    ) -> Result<SignUpResponse, BackendError> {
        self.post("account/signup", request).await
    }

    pub async fn post_forgot_password(
        &self,
        request: &ForgotPasswordRequest,
    ) -> Result<ForgotPasswordResponse, BackendError> {
        self.post("account/forgot-password", request).await
    }

    pub async fn post_reset_password(
        &self,
        request: &ResetPasswordRequest,
    ) -> Result<ResetPasswordResponse, BackendError> {
        self.post("account/reset-password", request).await
    }

    // Account

    pub async fn post_update_account(
        &self,
        token: &str,
        request: &UpdateAccountRequest,
    ) -> Result<UpdateAccountResponse, BackendError> {
        self.post_auth("account/update", token, request).await
    }

    pub async fn delete_account(
        &self,
        token: &str,
        user_id: &UserId,
    ) -> Result<DeleteAccountResponse, BackendError> {
        self.del_auth(&format!("account/{user_id}"), token).await
    }

    pub async fn get_discord_invite(
        &self,
        token: &str,
    ) -> Result<GetDiscordInviteResponse, BackendError> {
        self.get_auth("discord", token).await
    }

    // Characters

    pub async fn get_user_characters(
        &self,
        token: &str,
        user_id: &UserId,
    ) -> Result<GetUserCharactersResponse, BackendError> {
        self.get_auth(&format!("users/{user_id}/characters"), token)
            .await
    }

    pub async fn get_character_details(
        &self,
        token: &str,
        character_id: &UserCharacterId,
    ) -> Result<GetCharacterDetailsResponse, BackendError> {
        self.get_auth(&format!("characters/{character_id}"), token)
            .await
    }

    pub async fn get_character_by_name(
        &self,
        character_name: &str,
    ) -> Result<GetCharacterDetailsResponse, BackendError> {
        self.get(&format!("view-character/{character_name}")).await
    }

    pub async fn post_create_character(
        &self,
        token: &str,
        user_id: &UserId,
        request: &CreateCharacterRequest,
    ) -> Result<CreateCharacterResponse, BackendError> {
        self.post_auth(&format!("users/{user_id}/characters"), token, request)
            .await
    }

    pub async fn post_update_character(
        &self,
        token: &str,
        character_id: &UserCharacterId,
        request: &UpdateCharacterRequest,
    ) -> Result<UpdateAccountResponse, BackendError> {
        self.post_auth(&format!("characters/{character_id}"), token, request)
            .await
    }

    pub async fn delete_character(
        &self,
        token: &str,
        character_id: &UserCharacterId,
    ) -> Result<DeleteCharacterResponse, BackendError> {
        self.del_auth(&format!("characters/{character_id}"), token)
            .await
    }

    // Market

    pub async fn browse_market_items(
        &self,
        token: &str,
        request: &BrowseMarketItemsRequest,
    ) -> Result<BrowseMarketItemsResponse, BackendError> {
        self.post_auth("market", token, request).await
    }

    pub async fn buy_market_item(
        &self,
        token: &str,
        request: &BuyMarketItemRequest,
    ) -> Result<BuyMarketItemResponse, BackendError> {
        self.post_auth("market/buy", token, request).await
    }

    pub async fn reject_market_item(
        &self,
        token: &str,
        request: &RejectMarketItemRequest,
    ) -> Result<RejectMarketItemResponse, BackendError> {
        self.post_auth("market/reject", token, request).await
    }

    pub async fn sell_market_item(
        &self,
        token: &str,
        request: &SellMarketItemRequest,
    ) -> Result<SellMarketItemResponse, BackendError> {
        self.post_auth("market/sell", token, request).await
    }

    pub async fn edit_market_item(
        &self,
        token: &str,
        request: &EditMarketItemRequest,
    ) -> Result<EditMarketItemResponse, BackendError> {
        self.post_auth("market/edit", token, request).await
    }

    // Stash

    pub async fn upgrade_stash(
        &self,
        token: &str,
        request: &UpgradeStashRequest,
    ) -> Result<UpgradeStashResponse, BackendError> {
        self.post_auth("stashes/upgrade", token, request).await
    }

    pub async fn exchange_gems_stash(
        &self,
        token: &str,
        request: &ExchangeGemsStashRequest,
        stash_id: &StashId,
    ) -> Result<ExchangeGemsStashResponse, BackendError> {
        self.post_auth(&format!("stashes/{stash_id}/gems"), token, request)
            .await
    }

    pub async fn browse_stash_items(
        &self,
        token: &str,
        request: &BrowseStashItemsRequest,
        stash_id: &StashId,
    ) -> Result<BrowseStashItemsResponse, BackendError> {
        self.post_auth(&format!("stashes/{stash_id}"), token, request)
            .await
    }

    pub async fn take_stash_item(
        &self,
        token: &str,
        request: &TakeStashItemRequest,
        stash_id: &StashId,
    ) -> Result<TakeStashItemResponse, BackendError> {
        self.post_auth(&format!("stashes/{stash_id}/take"), token, request)
            .await
    }

    pub async fn store_stash_item(
        &self,
        token: &str,
        request: &StoreStashItemRequest,
        stash_id: &StashId,
    ) -> Result<StoreStashItemResponse, BackendError> {
        self.post_auth(&format!("stashes/{stash_id}/store"), token, request)
            .await
    }

    // Forge

    pub async fn forge_add_affix(
        &self,
        token: &str,
        request: &ForgeAddAffixRequest,
    ) -> Result<ForgeAddAffixResponse, BackendError> {
        self.post_auth("forge/add_affix", token, request).await
    }

    // Inventory

    pub async fn inventory_equip(
        &self,
        token: &str,
        request: &InventoryEquipRequest,
    ) -> Result<InventoryEquipResponse, BackendError> {
        self.post_auth("inventory/equip", token, request).await
    }

    pub async fn inventory_unequip(
        &self,
        token: &str,
        request: &InventoryUnequipRequest,
    ) -> Result<InventoryUnequipResponse, BackendError> {
        self.post_auth("inventory/unequip", token, request).await
    }

    pub async fn inventory_delete(
        &self,
        token: &str,
        request: &InventoryDeleteRequest,
    ) -> Result<InventoryDeleteResponse, BackendError> {
        self.post_auth("inventory/delete", token, request).await
    }

    // Protected

    async fn get<T>(&self, endpoint: &str) -> Result<T, BackendError>
    where
        T: serde::de::DeserializeOwned,
    {
        check_error(reqwest::get(format!("{}/{}", self.http_url, endpoint)).await).await
    }

    async fn get_auth<T>(&self, endpoint: &str, token: &str) -> Result<T, BackendError>
    where
        T: serde::de::DeserializeOwned,
    {
        if token.is_empty() {
            return Err(BackendError::Unauthorized("missing token".to_string()));
        }
        check_error(
            reqwest::Client::new()
                .get(format!("{}/{}", self.http_url, endpoint))
                .timeout(Duration::from_secs(60))
                .bearer_auth(token)
                .send()
                .await,
        )
        .await
    }

    async fn post<T, P>(&self, endpoint: &str, payload: &P) -> Result<T, BackendError>
    where
        T: serde::de::DeserializeOwned,
        P: serde::ser::Serialize,
    {
        check_error(
            reqwest::Client::new()
                .post(format!("{}/{}", self.http_url, endpoint))
                .timeout(Duration::from_secs(60))
                .json(payload)
                .send()
                .await,
        )
        .await
    }

    async fn post_auth<T, P>(
        &self,
        endpoint: &str,
        token: &str,
        payload: &P,
    ) -> Result<T, BackendError>
    where
        T: serde::de::DeserializeOwned,
        P: serde::ser::Serialize,
    {
        if token.is_empty() {
            return Err(BackendError::Unauthorized("missing token".to_string()));
        }
        check_error(
            reqwest::Client::new()
                .post(format!("{}/{}", self.http_url, endpoint))
                .bearer_auth(token)
                .timeout(Duration::from_secs(60))
                .json(payload)
                .send()
                .await,
        )
        .await
    }

    async fn del_auth<T>(&self, endpoint: &str, token: &str) -> Result<T, BackendError>
    where
        T: serde::de::DeserializeOwned,
    {
        if token.is_empty() {
            return Err(BackendError::Unauthorized("missing token".to_string()));
        }
        check_error(
            reqwest::Client::new()
                .delete(format!("{}/{}", self.http_url, endpoint))
                .timeout(Duration::from_secs(60))
                .bearer_auth(token)
                .send()
                .await,
        )
        .await
    }
}

async fn check_error<T>(response: reqwest::Result<reqwest::Response>) -> Result<T, BackendError>
where
    T: serde::de::DeserializeOwned,
{
    match response {
        Ok(response) => {
            let response_status = response.status();
            match response_status {
                StatusCode::OK => match response.json().await {
                    Ok(response) => Ok(response),
                    Err(_) => Err(BackendError::OtherError),
                },
                _ => {
                    let reason = response
                        .json::<ErrorResponse>()
                        .await
                        .unwrap_or_default()
                        .error;
                    Err(match response_status {
                        StatusCode::NOT_FOUND => BackendError::NotFound,
                        StatusCode::UNAUTHORIZED => BackendError::Unauthorized(reason),
                        StatusCode::FORBIDDEN => BackendError::Forbidden,
                        StatusCode::CONFLICT => BackendError::UserError(reason),
                        StatusCode::INTERNAL_SERVER_ERROR => BackendError::ServerError(reason),
                        _ => BackendError::OtherError,
                    })
                }
            }
        }
        Err(_) => Err(BackendError::ServerNotResponding),
    }
}
