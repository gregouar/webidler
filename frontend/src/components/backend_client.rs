use anyhow::Result;
use reqwest::StatusCode;
use std::time::Duration;

use shared::{
    data::user::UserId,
    http::{
        client::{SignInRequest, SignUpRequest},
        server::{
            ErrorResponse, GetUserCharactersResponse, GetUserResponse, LeaderboardResponse,
            PlayersCountResponse, SignInResponse, SignUpResponse, SkillsResponse,
        },
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

    pub async fn get_me(&self, token: &str) -> Result<GetUserResponse> {
        self.get_auth(&format!("account/me"), token).await
    }

    pub async fn get_characters(
        &self,
        token: &str,
        user_id: &UserId,
    ) -> Result<GetUserCharactersResponse> {
        self.get_auth(&format!("users/{user_id}/characters"), token)
            .await
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
        check_error(reqwest::get(format!("{}/{}", self.http_url, endpoint)).await?).await
    }

    async fn get_auth<T>(&self, endpoint: &str, token: &str) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        check_error(
            reqwest::Client::new()
                .get(format!("{}/{}", self.http_url, endpoint))
                .bearer_auth(token)
                .send()
                .await?,
        )
        .await
    }

    async fn post<T, P>(&self, endpoint: &str, payload: &P) -> Result<T>
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
                .await?,
        )
        .await
    }
}

async fn check_error<T>(response: reqwest::Response) -> Result<T>
where
    T: serde::de::DeserializeOwned,
{
    match response.status() {
        StatusCode::OK => Ok(response.json().await?),
        _ => Err(anyhow::anyhow!(
            "{}",
            response.json::<ErrorResponse>().await?
        )),
    }
}
