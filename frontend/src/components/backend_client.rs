use anyhow::Result;

use shared::http::server::{LeaderboardResponse, PlayersCountResponse, SkillsResponse};

#[derive(Clone, Copy)]
pub struct BackendClient {
    http_url: &'static str,
    ws_url: &'static str,
}

impl BackendClient {
    pub fn new(http_url: &'static str, ws_url: &'static str) -> Self {
        BackendClient {
            http_url: http_url.trim_end_matches('/').into(),
            ws_url: ws_url.trim_end_matches('/').into(),
        }
    }

    pub fn get_game_ws_url(&self) -> String {
        format!("{}/ws", self.ws_url)
    }

    pub async fn get<T>(&self, endoint: &str) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        Ok(serde_json::from_str(
            &reqwest::get(format!("{}/{}", self.http_url, endoint))
                .await?
                .error_for_status()?
                .text()
                .await?,
        )?)
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
}
