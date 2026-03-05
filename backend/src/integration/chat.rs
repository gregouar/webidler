use std::{env, sync::Arc};

use shared::data::item::ItemSpecs;
use shared_chat::{
    messages::client::ClientPostMessage,
    types::{ChatChannel, ChatContent, LinkedItemBytes, UserId},
};

#[derive(Clone)]
pub struct ChatIntegration {
    pub chat_url: Arc<String>,
}

impl ChatIntegration {
    pub fn from_env() -> Self {
        Self {
            chat_url: Arc::new(env::var("CHAT_URL").expect("CHAT_URL must be set")),
        }
    }

    pub async fn send_private_message(
        &self,
        user_id: UserId,
        content: String,
        linked_item: Option<&ItemSpecs>,
    ) -> anyhow::Result<()> {
        let res = reqwest::Client::new()
            .post(format!("{}/message/{}", self.chat_url, user_id))
            .header("Content-Type", "application/json")
            .json(&ClientPostMessage {
                channel: ChatChannel::System,
                content: ChatContent::try_new(content)?,
                linked_item: linked_item.and_then(to_linked_item_bytes),
            })
            .send()
            .await?;

        if !res.status().is_success() {
            let err = res.text().await?;
            anyhow::bail!("Chat API error: {}", err);
        }

        Ok(())
    }

    pub async fn broadcast_message(
        &self,
        content: String,
        linked_item: Option<&ItemSpecs>,
    ) -> anyhow::Result<()> {
        let res = reqwest::Client::new()
            .post(format!("{}/message", self.chat_url))
            .header("Content-Type", "application/json")
            .json(&ClientPostMessage {
                channel: ChatChannel::System,
                content: ChatContent::try_new(content)?,
                linked_item: linked_item.and_then(to_linked_item_bytes),
            })
            .send()
            .await?;

        if !res.status().is_success() {
            let err = res.text().await?;
            anyhow::bail!("Chat API error: {}", err);
        }

        Ok(())
    }
}

fn to_linked_item_bytes(item_specs: &ItemSpecs) -> Option<LinkedItemBytes> {
    // let mut cloned_item_specs = item_specs.clone();
    // cloned_item_specs.signature = Default::default();
    LinkedItemBytes::try_new(rmp_serde::to_vec(item_specs).ok()?).ok()
}
