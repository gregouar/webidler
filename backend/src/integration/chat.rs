use shared::data::item::ItemSpecs;
use shared_chat::{
    messages::client::ClientPostMessage,
    types::{ChatChannel, ChatContent, UserId},
};

use crate::app_state::AppSettings;

pub async fn send_private_message(
    app_settings: &AppSettings,
    user_id: UserId,
    content: String,
    linked_item: Option<&ItemSpecs>,
) -> anyhow::Result<()> {
    let res = reqwest::Client::new()
        .post(format!("{}/message/{}", app_settings.chat_url, user_id))
        .header("Content-Type", "application/json")
        .json(&ClientPostMessage {
            channel: ChatChannel::System,
            content: ChatContent::try_new(content)?,
            linked_item: linked_item.and_then(|item_specs| rmp_serde::to_vec(item_specs).ok()),
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
    app_settings: &AppSettings,
    content: String,
    linked_item: Option<&ItemSpecs>,
) -> anyhow::Result<()> {
    let res = reqwest::Client::new()
        .post(format!("{}/message", app_settings.chat_url))
        .header("Content-Type", "application/json")
        .json(&ClientPostMessage {
            channel: ChatChannel::System,
            content: ChatContent::try_new(content)?,
            linked_item: linked_item.and_then(|item_specs| rmp_serde::to_vec(item_specs).ok()),
        })
        .send()
        .await?;

    if !res.status().is_success() {
        let err = res.text().await?;
        anyhow::bail!("Chat API error: {}", err);
    }

    Ok(())
}
