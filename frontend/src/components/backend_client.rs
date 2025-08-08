use std::time::Duration;

use anyhow::Result;

use shared::http::{
    client::{SignInRequest, SignUpRequest},
    server::{
        GetUserResponse, LeaderboardResponse, PlayersCountResponse, SignInResponse, SignUpResponse,
        SkillsResponse,
    },
};

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

    pub fn get_game_ws_url(&self) -> String {
        format!("{}/ws", self.ws_url)
    }

    pub async fn get_players_count(&self) -> Result<PlayersCountResponse> {
        self.get("players").await
    }

    pub async fn get_leaderboard(&self) -> Result<LeaderboardResponse> {
        self.get("leaderboard").await
    }

    pub async fn get_skills(&self) -> Result<SkillsResponse> {
        self.get("game/skills").await
    }

    pub async fn get_user(&self, jwt: String, user_id: &str) -> Result<GetUserResponse> {
        self.get(&format!("account/users/{user_id}")).await
    }

    pub async fn post_signin(&self, request: &SignInRequest) -> Result<SignInResponse> {
        self.post("account/signin", request).await
    }

    pub async fn post_signup(&self, request: &SignUpRequest) -> Result<SignUpResponse> {
        self.post("account/signup", request).await
    }

    async fn get<T>(&self, endpoint: &str) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        Ok(reqwest::get(format!("{}/{}", self.http_url, endpoint))
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    async fn post<T, P>(&self, endpoint: &str, payload: &P) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
        P: serde::ser::Serialize,
    {
        Ok(reqwest::Client::new()
            .post(format!("{}/{}", self.http_url, endpoint))
            .timeout(Duration::from_secs(60))
            .json(payload)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }
}
