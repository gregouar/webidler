use anyhow::Result;

use shared::http::server::{LeaderboardResponse, PlayersCountResponse, SkillsResponse};

#[derive(Clone)]
pub struct RestContext {
    host: String,
}

impl RestContext {
    pub fn new(host: &str) -> Self {
        RestContext {
            host: host.trim_end_matches('/').to_string(),
        }
    }

    pub async fn get<T>(&self, endoint: &str) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        Ok(serde_json::from_str(
            &reqwest::get(format!("{}/{}", self.host, endoint))
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
