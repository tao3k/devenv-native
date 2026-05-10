//! Discord bot identity hydration and test identity helpers.

use anyhow::Context;
use serde::Deserialize;
use std::sync::PoisonError;

use super::state::DiscordChannel;

#[derive(Debug, Deserialize)]
struct DiscordCurrentUserPayload {
    id: String,
}

/// Discord bot user identifier used by mention parsing tests.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscordBotUserId(String);

impl DiscordBotUserId {
    /// Build a bot user identifier.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl From<String> for DiscordBotUserId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&str> for DiscordBotUserId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl DiscordChannel {
    pub(in super::super) async fn hydrate_bot_identity(&self) -> anyhow::Result<()> {
        let url = self.api_url("users/@me");
        let response = self
            .client
            .get(url)
            .header("Authorization", format!("Bot {}", self.bot_token))
            .send()
            .await
            .context("discord bot identity request failed")?;
        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            let preview = body.chars().take(256).collect::<String>();
            anyhow::bail!("discord bot identity failed: status={status} body={preview}");
        }
        let payload: DiscordCurrentUserPayload = response
            .json()
            .await
            .context("failed to decode discord bot identity response")?;
        self.set_bot_user_id(Some(payload.id));
        Ok(())
    }

    pub(in super::super) fn set_bot_user_id(&self, bot_user_id: Option<String>) {
        *self
            .bot_user_id
            .write()
            .unwrap_or_else(PoisonError::into_inner) = bot_user_id.and_then(|value| {
            let trimmed = value.trim().to_string();
            (!trimmed.is_empty()).then_some(trimmed)
        });
    }

    pub(in super::super) fn bot_user_id(&self) -> Option<String> {
        self.bot_user_id
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    pub(in super::super) fn is_bot_user_id(&self, candidate: Option<&str>) -> bool {
        let Some(expected) = self.bot_user_id() else {
            return false;
        };
        candidate.is_some_and(|value| value.trim() == expected)
    }

    #[doc(hidden)]
    pub fn set_bot_user_id_for_tests(&self, bot_user_id: Option<DiscordBotUserId>) {
        self.set_bot_user_id(bot_user_id.map(|value| value.0));
    }
}
