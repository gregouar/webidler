use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use anyhow::{Context, Result};
use backend_shared::http::users::UserId;
use serde::Deserialize;
use shared_chat::types::CharacterId;

const REFRESH_COOLDOWN: Duration = Duration::from_secs(10);

pub struct CharacterResolver {
    http_client: reqwest::Client,
    backend_url: String,
    user_id: UserId,
    characters: HashMap<CharacterId, String>,
    last_refresh_attempt: Option<Instant>,
}

impl CharacterResolver {
    pub async fn connect(backend_url: &str, user_id: UserId) -> Result<Self> {
        let mut resolver = Self {
            http_client: reqwest::Client::new(),
            backend_url: backend_url.trim_end_matches('/').to_string(),
            user_id,
            characters: HashMap::new(),
            last_refresh_attempt: None,
        };
        resolver.characters = resolver.fetch_characters().await?;
        Ok(resolver)
    }

    pub async fn resolve(&mut self, character_id: Option<CharacterId>) -> Result<Option<String>> {
        let Some(character_id) = character_id else {
            return Ok(None);
        };

        if let Some(character_name) = self.characters.get(&character_id) {
            return Ok(Some(character_name.clone()));
        }

        if !self.can_refresh() {
            anyhow::bail!("invalid character");
        }

        // Record attempts before doing I/O so failures are rate limited as well.
        self.last_refresh_attempt = Some(Instant::now());
        self.characters = self
            .fetch_characters()
            .await
            .context("failed to refresh characters")?;

        self.characters
            .get(&character_id)
            .cloned()
            .map(Some)
            .ok_or_else(|| anyhow::anyhow!("invalid character"))
    }

    fn can_refresh(&self) -> bool {
        self.last_refresh_attempt
            .is_none_or(|last_attempt| last_attempt.elapsed() >= REFRESH_COOLDOWN)
    }

    async fn fetch_characters(&self) -> Result<HashMap<CharacterId, String>> {
        let res = self
            .http_client
            .get(format!(
                "{}/users/{}/characters",
                self.backend_url, self.user_id
            ))
            .header("Content-Type", "application/json")
            .send()
            .await?;

        if !res.status().is_success() {
            let err = res.text().await?;
            anyhow::bail!("Server API error: {}", err);
        }

        Ok(res
            .json::<GetUserCharactersResponse>()
            .await?
            .characters
            .into_iter()
            .map(|character| (character.character_id, character.name))
            .collect())
    }
}

#[derive(Deserialize)]
struct GetUserCharactersResponse {
    characters: Vec<UserCharacter>,
}

#[derive(Deserialize)]
struct UserCharacter {
    character_id: CharacterId,
    name: String,
}
