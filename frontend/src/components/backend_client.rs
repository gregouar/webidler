use codee::{Decoder, binary::MsgpackSerdeCodec};
use leptos::prelude::*;
use reqwest::StatusCode;
use std::time::Duration;

use shared::{
    data::{
        realms::Realm,
        stash::StashId,
        user::{UserCharacterId, UserId},
    },
    http::{
        client::{
            AscendPassivesRequest, BrowseMarketItemsRequest, BrowseStashItemsRequest,
            BuyBenedictionsRequest, BuyMarketItemRequest, CreateCharacterRequest,
            EditMarketItemRequest, ExchangeGemsStashRequest, ForgeAffixRequest,
            ForgeUpgradeRequest, ForgotPasswordRequest, GambleItemRequest, InventoryDeleteRequest,
            InventoryEquipRequest, InventoryUnequipRequest, RejectMarketItemRequest,
            ResetPasswordRequest, SavePassivesRequest, SaveSkillMasteriesRequest,
            SellMarketItemRequest, SignInRequest, SignUpRequest, SocketPassiveRequest,
            StoreStashItemRequest, TakeStashItemRequest, UpdateAccountRequest,
            UpdateCharacterRequest, UpgradeStashRequest,
        },
        server::{
            AscendPassivesResponse, BrowseMarketItemsResponse, BrowseStashItemsResponse,
            BuyBenedictionsResponse, BuyMarketItemResponse, CreateCharacterResponse,
            DeleteAccountResponse, DeleteCharacterResponse, EditMarketItemResponse, ErrorResponse,
            ExchangeGemsStashResponse, ForgeAffixResponse, ForgeUpgradeResponse,
            ForgotPasswordResponse, GambleItemResponse, GetAreasResponse, GetBenedictionsResponse,
            GetCharacterDetailsResponse, GetDiscordInviteResponse, GetPassivesResponse,
            GetSkillsResponse, GetStatusesResponse, GetUserCharactersResponse,
            GetUserDetailsResponse, InventoryDeleteResponse, InventoryEquipResponse,
            InventoryUnequipResponse, LeaderboardResponse, NewsResponse, PlayersCountResponse,
            RejectMarketItemResponse, ResetPasswordResponse, SavePassivesResponse,
            SaveSkillMasteriesResponse, SellMarketItemResponse, SignInResponse, SignUpResponse,
            SocketPassiveResponse, StoreStashItemResponse, TakeStashItemResponse,
            UpdateAccountResponse, UpgradeStashResponse,
        },
    },
};

use crate::components::auth::AuthState;

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
    auth: AuthState,
}

impl BackendClient {
    pub fn new(http_url: &'static str, ws_url: &'static str) -> Self {
        BackendClient {
            http_url: http_url.trim_end_matches('/'),
            ws_url: ws_url.trim_end_matches('/'),
            auth: AuthState::new(),
        }
    }

    pub fn sign_out(&self) {
        self.auth.sign_out();
    }

    pub fn is_authenticated(&self) -> bool {
        self.auth.is_authenticated()
    }
    pub fn track_authenticated(&self) -> bool {
        self.auth.track_authenticated()
    }

    pub async fn get_access_token(&self) -> Result<String, BackendError> {
        self.auth.get_access_token(*self).await
    }

    // Game

    pub fn get_game_ws_url(&self) -> String {
        format!("{}/ws", self.ws_url)
    }

    pub async fn get_players_count(&self) -> Result<PlayersCountResponse, BackendError> {
        self.get("players").await
    }

    pub async fn get_leaderboard(&self, realm: Realm) -> Result<LeaderboardResponse, BackendError> {
        self.get(&format!("leaderboard?realm={}", realm.realm_id()))
            .await
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

    pub async fn get_statuses(&self) -> Result<GetStatusesResponse, BackendError> {
        self.get("game/statuses").await
    }

    pub async fn get_passives(&self) -> Result<GetPassivesResponse, BackendError> {
        self.get("game/passives").await
    }

    pub async fn post_ascend_passives(
        &self,
        request: &AscendPassivesRequest,
    ) -> Result<AscendPassivesResponse, BackendError> {
        self.post_auth("game/passives", request).await
    }

    pub async fn post_socket_passive(
        &self,
        request: &SocketPassiveRequest,
    ) -> Result<SocketPassiveResponse, BackendError> {
        self.post_auth("game/passives/socket", request).await
    }

    pub async fn post_save_passives(
        &self,
        request: &SavePassivesRequest,
    ) -> Result<SavePassivesResponse, BackendError> {
        self.post_auth("game/passives/build", request).await
    }

    pub async fn get_benedictions(&self) -> Result<GetBenedictionsResponse, BackendError> {
        self.get("game/benedictions").await
    }

    pub async fn post_buy_benedictions(
        &self,
        request: &BuyBenedictionsRequest,
    ) -> Result<BuyBenedictionsResponse, BackendError> {
        self.post_auth("game/benedictions", request).await
    }

    pub async fn post_save_skill_masteries(
        &self,
        request: &SaveSkillMasteriesRequest,
    ) -> Result<SaveSkillMasteriesResponse, BackendError> {
        self.post_auth("game/skill-masteries", request).await
    }

    // Auth

    pub async fn get_me(&self) -> Result<GetUserDetailsResponse, BackendError> {
        self.get_auth("account/me").await
    }

    pub async fn post_signin(
        &self,
        request: &SignInRequest,
    ) -> Result<SignInResponse, BackendError> {
        let response: SignInResponse = self.post("account/signin", request).await?;
        self.auth.set_access_token(response.jwt.clone());
        Ok(response)
    }

    pub async fn post_refresh(&self) -> Result<SignInResponse, BackendError> {
        self.post("account/refresh", &()).await
    }

    pub async fn post_signout(&self) -> Result<SignInResponse, BackendError> {
        self.post("account/signout", &()).await
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
        request: &UpdateAccountRequest,
    ) -> Result<UpdateAccountResponse, BackendError> {
        self.post_auth("account/update", request).await
    }

    pub async fn delete_account(
        &self,
        user_id: &UserId,
    ) -> Result<DeleteAccountResponse, BackendError> {
        self.del_auth(&format!("account/{user_id}")).await
    }

    pub async fn get_discord_invite(&self) -> Result<GetDiscordInviteResponse, BackendError> {
        self.get_auth("discord").await
    }

    // Characters

    pub async fn get_user_characters(
        &self,
        user_id: &UserId,
    ) -> Result<GetUserCharactersResponse, BackendError> {
        self.get_auth(&format!("users/{user_id}/characters")).await
    }

    pub async fn get_character_details(
        &self,
        character_id: &UserCharacterId,
    ) -> Result<GetCharacterDetailsResponse, BackendError> {
        self.get_auth(&format!("characters/{character_id}")).await
    }

    pub async fn get_character_by_name(
        &self,
        character_name: &str,
    ) -> Result<GetCharacterDetailsResponse, BackendError> {
        self.get(&format!("view-character/{character_name}")).await
    }

    pub async fn post_create_character(
        &self,
        user_id: &UserId,
        request: &CreateCharacterRequest,
    ) -> Result<CreateCharacterResponse, BackendError> {
        self.post_auth(&format!("users/{user_id}/characters"), request)
            .await
    }

    pub async fn post_update_character(
        &self,
        character_id: &UserCharacterId,
        request: &UpdateCharacterRequest,
    ) -> Result<UpdateAccountResponse, BackendError> {
        self.post_auth(&format!("characters/{character_id}"), request)
            .await
    }

    pub async fn delete_character(
        &self,
        character_id: &UserCharacterId,
    ) -> Result<DeleteCharacterResponse, BackendError> {
        self.del_auth(&format!("characters/{character_id}")).await
    }

    // Market

    pub async fn browse_market_items(
        &self,
        request: &BrowseMarketItemsRequest,
    ) -> Result<BrowseMarketItemsResponse, BackendError> {
        self.post_auth("market", request).await
    }

    pub async fn buy_market_item(
        &self,
        request: &BuyMarketItemRequest,
    ) -> Result<BuyMarketItemResponse, BackendError> {
        self.post_auth("market/buy", request).await
    }

    pub async fn reject_market_item(
        &self,
        request: &RejectMarketItemRequest,
    ) -> Result<RejectMarketItemResponse, BackendError> {
        self.post_auth("market/reject", request).await
    }

    pub async fn sell_market_item(
        &self,
        request: &SellMarketItemRequest,
    ) -> Result<SellMarketItemResponse, BackendError> {
        self.post_auth("market/sell", request).await
    }

    pub async fn edit_market_item(
        &self,
        request: &EditMarketItemRequest,
    ) -> Result<EditMarketItemResponse, BackendError> {
        self.post_auth("market/edit", request).await
    }

    // Stash

    pub async fn upgrade_stash(
        &self,
        request: &UpgradeStashRequest,
    ) -> Result<UpgradeStashResponse, BackendError> {
        self.post_auth("stashes/upgrade", request).await
    }

    pub async fn exchange_gems_stash(
        &self,
        request: &ExchangeGemsStashRequest,
        stash_id: &StashId,
    ) -> Result<ExchangeGemsStashResponse, BackendError> {
        self.post_auth(&format!("stashes/{stash_id}/gems"), request)
            .await
    }

    pub async fn browse_stash_items(
        &self,
        request: &BrowseStashItemsRequest,
        stash_id: &StashId,
    ) -> Result<BrowseStashItemsResponse, BackendError> {
        self.post_auth(&format!("stashes/{stash_id}"), request)
            .await
    }

    pub async fn take_stash_item(
        &self,
        request: &TakeStashItemRequest,
        stash_id: &StashId,
    ) -> Result<TakeStashItemResponse, BackendError> {
        self.post_auth(&format!("stashes/{stash_id}/take"), request)
            .await
    }

    pub async fn store_stash_item(
        &self,
        request: &StoreStashItemRequest,
        stash_id: &StashId,
    ) -> Result<StoreStashItemResponse, BackendError> {
        self.post_auth(&format!("stashes/{stash_id}/store"), request)
            .await
    }

    // Forge

    pub async fn forge_affix(
        &self,
        request: &ForgeAffixRequest,
    ) -> Result<ForgeAffixResponse, BackendError> {
        self.post_auth("forge/affix", request).await
    }

    pub async fn forge_upgrade(
        &self,
        request: &ForgeUpgradeRequest,
    ) -> Result<ForgeUpgradeResponse, BackendError> {
        self.post_auth("forge/upgrade", request).await
    }

    pub async fn gamble_item(
        &self,
        request: &GambleItemRequest,
    ) -> Result<GambleItemResponse, BackendError> {
        self.post_auth("forge/gamble", request).await
    }

    // Inventory

    pub async fn inventory_equip(
        &self,
        request: &InventoryEquipRequest,
    ) -> Result<InventoryEquipResponse, BackendError> {
        self.post_auth("inventory/equip", request).await
    }

    pub async fn inventory_unequip(
        &self,
        request: &InventoryUnequipRequest,
    ) -> Result<InventoryUnequipResponse, BackendError> {
        self.post_auth("inventory/unequip", request).await
    }

    pub async fn inventory_delete(
        &self,
        request: &InventoryDeleteRequest,
    ) -> Result<InventoryDeleteResponse, BackendError> {
        self.post_auth("inventory/delete", request).await
    }

    // Protected

    async fn get<T>(&self, endpoint: &str) -> Result<T, BackendError>
    where
        T: serde::de::DeserializeOwned,
    {
        deserialize_response(reqwest::get(format!("{}/{}", self.http_url, endpoint)).await).await
    }

    async fn get_auth<T>(&self, endpoint: &str) -> Result<T, BackendError>
    where
        T: serde::de::DeserializeOwned,
    {
        let token = self.get_access_token().await?;
        deserialize_response(
            reqwest::Client::new()
                .get(format!("{}/{}", self.http_url, endpoint))
                .timeout(Duration::from_secs(60))
                .bearer_auth(&token)
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
        deserialize_response(
            reqwest::Client::new()
                .post(format!("{}/{}", self.http_url, endpoint))
                .timeout(Duration::from_secs(60))
                .json(payload)
                .with_credentials()
                .send()
                .await,
        )
        .await
    }

    async fn post_auth<T, P>(&self, endpoint: &str, payload: &P) -> Result<T, BackendError>
    where
        T: serde::de::DeserializeOwned,
        P: serde::ser::Serialize,
    {
        let token = self.get_access_token().await?;
        deserialize_response(
            reqwest::Client::new()
                .post(format!("{}/{}", self.http_url, endpoint))
                .bearer_auth(&token)
                .timeout(Duration::from_secs(60))
                .json(payload)
                .send()
                .await,
        )
        .await
    }

    async fn del_auth<T>(&self, endpoint: &str) -> Result<T, BackendError>
    where
        T: serde::de::DeserializeOwned,
    {
        let token = self.get_access_token().await?;
        deserialize_response(
            reqwest::Client::new()
                .delete(format!("{}/{}", self.http_url, endpoint))
                .timeout(Duration::from_secs(60))
                .bearer_auth(&token)
                .send()
                .await,
        )
        .await
    }
}

trait RequestBuilderCredentialsExt {
    fn with_credentials(self) -> Self;
}

impl RequestBuilderCredentialsExt for reqwest::RequestBuilder {
    #[cfg(target_arch = "wasm32")]
    fn with_credentials(self) -> Self {
        self.fetch_credentials_include()
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn with_credentials(self) -> Self {
        self
    }
}

async fn deserialize_response<T>(
    response: reqwest::Result<reqwest::Response>,
) -> Result<T, BackendError>
where
    T: serde::de::DeserializeOwned,
{
    let Ok(response) = response else {
        return Err(BackendError::ServerNotResponding);
    };

    let status = response.status();
    if status != StatusCode::OK {
        let err = match response.json::<ErrorResponse>().await {
            Ok(e) => e.error,
            Err(_) => String::new(),
        };

        return Err(match status {
            StatusCode::NOT_FOUND => BackendError::NotFound,
            StatusCode::UNAUTHORIZED => BackendError::Unauthorized(err),
            StatusCode::FORBIDDEN => BackendError::Forbidden,
            StatusCode::CONFLICT => BackendError::UserError(err),
            StatusCode::INTERNAL_SERVER_ERROR => BackendError::ServerError(err),
            _ => BackendError::OtherError,
        });
    }

    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if content_type.contains("msgpack") {
        let bytes = response
            .bytes()
            .await
            .map_err(|_| BackendError::OtherError)?;

        MsgpackSerdeCodec::decode(&bytes).map_err(|_| BackendError::OtherError)
    } else {
        response.json().await.map_err(|_| BackendError::OtherError)
    }
}
