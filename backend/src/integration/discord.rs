use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use chrono::{DateTime, Duration, Utc};
use serde::Serialize;
use shared::data::user::UserId;

#[derive(Clone)]
pub struct DiscordInvitesStore {
    discord_bot_token: Arc<String>,
    invites_cache: Arc<Mutex<HashMap<UserId, Arc<tokio::sync::Mutex<InviteCache>>>>>,
}

impl DiscordInvitesStore {
    pub fn new(discord_bot_token: String) -> Self {
        Self {
            discord_bot_token: Arc::new(discord_bot_token),
            invites_cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn get_invite(&self, user_id: UserId) -> anyhow::Result<String> {
        let cached_invite = {
            self.invites_cache
                .lock()
                .unwrap()
                .entry(user_id)
                .or_default()
                .clone()
        };
        let mut cached_invite = cached_invite.lock().await;

        let now = Utc::now();
        if cached_invite.expires_at < now {
            cached_invite.code = generate_discord_invite(&self.discord_bot_token).await?;
            cached_invite.expires_at = now + Duration::hours(23);
        }

        Ok(cached_invite.code.clone())
    }
}

#[derive(Clone, Default)]
struct InviteCache {
    expires_at: DateTime<Utc>,
    code: String,
}

#[derive(Serialize)]
struct DiscordCreateInviteBody {
    max_age: u64,
    max_uses: u64,
    temporary: bool,
    unique: bool,
}

async fn generate_discord_invite(discord_bot_token: &str) -> anyhow::Result<String> {
    let body = DiscordCreateInviteBody {
        max_age: 86400, // 24 hours
        max_uses: 1,
        temporary: false,
        unique: true,
    };

    let res = reqwest::Client::new()
        .post("https://discord.com/api/v10/channels/734765714497601649/invites")
        .header("Authorization", format!("Bot {}", discord_bot_token))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await?;

    if !res.status().is_success() {
        let err = res.text().await?;
        anyhow::bail!("Discord API error: {}", err);
    }

    res
        .json::<serde_json::Value>()
        .await?
        .get("code")
        .and_then(|code| code.as_str())
        .map(|code| code.to_string())
        .ok_or(anyhow::anyhow!("failed to get discord invite"))
}
