use anyhow::{Context, Result};
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use backend_shared::http::users::{CharacterCosmetics, GetUserCharactersResponse, UserId};
use shared_chat::types::CharacterId;

const REFRESH_COOLDOWN: Duration = Duration::from_secs(10);

pub struct CharacterResolver {
    http_client: reqwest::Client,
    backend_url: String,
    user_id: UserId,
    characters: HashMap<CharacterId, CharacterPresentation>,
    last_refresh_attempt: Option<Instant>,
}

#[derive(Clone)]
pub struct CharacterPresentation {
    pub name: String,
    pub cosmetics: CharacterCosmetics,
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

    pub async fn resolve(
        &mut self,
        character_id: Option<CharacterId>,
    ) -> Result<Option<CharacterPresentation>> {
        let Some(character_id) = character_id else {
            return Ok(None);
        };

        if self.can_refresh() {
            self.last_refresh_attempt = Some(Instant::now());
            if let Ok(characters) = self.fetch_characters().await {
                self.characters = characters;
            }
        }

        if let Some(character) = self.characters.get(&character_id) {
            return Ok(Some(character.clone()));
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

    async fn fetch_characters(&self) -> Result<HashMap<CharacterId, CharacterPresentation>> {
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
            .map(|character| {
                (
                    character.character_id,
                    CharacterPresentation {
                        name: character.name,
                        cosmetics: character.cosmetics,
                    },
                )
            })
            .collect())
    }
}
