use std::time::Duration;

use reqwest::{Response, StatusCode};
use serde_json::Value;
use tokio::sync::mpsc;

use crate::channels::traits::ChannelMessage;

use super::TelegramChannel;
use super::constants::{
    TELEGRAM_POLL_CONFLICT_RETRY_SECS, TELEGRAM_POLL_DEFAULT_RATE_LIMIT_RETRY_SECS,
    TELEGRAM_POLL_MAX_RATE_LIMIT_RETRY_SECS, TELEGRAM_POLL_RETRY_SECS,
};
use super::error::{
    telegram_api_error_code, telegram_api_error_description, telegram_api_error_retry_after_secs,
};

enum PollUpdatesOutcome {
    Continue,
    Updates(Value),
}

impl TelegramChannel {
    pub(super) async fn listen_updates(
        &self,
        tx: mpsc::Sender<ChannelMessage>,
    ) -> anyhow::Result<()> {
        let mut offset: i64 = 0;
        tracing::info!("Telegram channel listening for messages...");
        loop {
            match self.poll_updates(offset).await? {
                PollUpdatesOutcome::Continue => {}
                PollUpdatesOutcome::Updates(data) => {
                    if self.process_update_batch(&tx, &data, &mut offset).await? {
                        return Ok(());
                    }
                }
            }
        }
    }

    pub(super) async fn health_probe(&self) -> bool {
        match tokio::time::timeout(
            Duration::from_secs(5),
            self.client.get(self.api_url("getMe")).send(),
        )
        .await
        {
            Ok(Ok(resp)) => resp.status().is_success(),
            Ok(Err(_)) | Err(_) => false,
        }
    }

    async fn poll_updates(&self, offset: i64) -> anyhow::Result<PollUpdatesOutcome> {
        let url = self.api_url("getUpdates");
        let body = serde_json::json!({
            "offset": offset,
            "timeout": 30,
            "allowed_updates": ["message"]
        });
        let resp = match self.client.post(&url).json(&body).send().await {
            Ok(resp) => resp,
            Err(error) => {
                tracing::warn!("Telegram poll error: {error}");
                tokio::time::sleep(Duration::from_secs(TELEGRAM_POLL_RETRY_SECS)).await;
                return Ok(PollUpdatesOutcome::Continue);
            }
        };

        if !resp.status().is_success() {
            return Self::handle_http_poll_error(resp).await;
        }

        let data: Value = match resp.json().await {
            Ok(data) => data,
            Err(error) => {
                tracing::warn!("Telegram parse error: {error}");
                tokio::time::sleep(Duration::from_secs(TELEGRAM_POLL_RETRY_SECS)).await;
                return Ok(PollUpdatesOutcome::Continue);
            }
        };

        if !data.get("ok").and_then(Value::as_bool).unwrap_or(true) {
            return Self::handle_api_poll_error(&data).await;
        }

        Ok(PollUpdatesOutcome::Updates(data))
    }

    async fn handle_http_poll_error(resp: Response) -> anyhow::Result<PollUpdatesOutcome> {
        let http_status = resp.status();
        let body_text = resp.text().await.unwrap_or_default();
        let maybe_data = serde_json::from_str::<Value>(&body_text).ok();
        let description = maybe_data.as_ref().map_or(body_text.as_str(), |data| {
            telegram_api_error_description(data, body_text.as_str())
        });

        match http_status {
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
                anyhow::bail!(
                    "Telegram getUpdates HTTP error (status={http_status}): {description}"
                );
            }
            StatusCode::CONFLICT => {
                tracing::warn!(
                    "Telegram polling conflict (HTTP 409): {description}. \
Ensure only one process is using this bot token."
                );
                tokio::time::sleep(Duration::from_secs(TELEGRAM_POLL_CONFLICT_RETRY_SECS)).await;
            }
            StatusCode::TOO_MANY_REQUESTS => {
                let retry_after_secs = maybe_data
                    .as_ref()
                    .and_then(telegram_api_error_retry_after_secs)
                    .unwrap_or(TELEGRAM_POLL_DEFAULT_RATE_LIMIT_RETRY_SECS)
                    .clamp(1, TELEGRAM_POLL_MAX_RATE_LIMIT_RETRY_SECS);
                tracing::warn!(
                    retry_after_secs,
                    "Telegram getUpdates HTTP 429 rate limited: {description}"
                );
                tokio::time::sleep(Duration::from_secs(retry_after_secs)).await;
            }
            _ => {
                tracing::warn!(
                    status = %http_status,
                    "Telegram getUpdates HTTP error: {description}"
                );
                tokio::time::sleep(Duration::from_secs(TELEGRAM_POLL_RETRY_SECS)).await;
            }
        }

        Ok(PollUpdatesOutcome::Continue)
    }

    async fn handle_api_poll_error(data: &Value) -> anyhow::Result<PollUpdatesOutcome> {
        let error_code = telegram_api_error_code(data).unwrap_or_default();
        let description = telegram_api_error_description(data, "unknown Telegram API error");

        match error_code {
            401 | 403 => {
                anyhow::bail!("Telegram getUpdates API error (code={error_code}): {description}");
            }
            409 => {
                tracing::warn!(
                    "Telegram polling conflict (409): {description}. \
Ensure only one process is using this bot token."
                );
                tokio::time::sleep(Duration::from_secs(TELEGRAM_POLL_CONFLICT_RETRY_SECS)).await;
            }
            429 => {
                let retry_after_secs = telegram_api_error_retry_after_secs(data)
                    .unwrap_or(TELEGRAM_POLL_DEFAULT_RATE_LIMIT_RETRY_SECS)
                    .clamp(1, TELEGRAM_POLL_MAX_RATE_LIMIT_RETRY_SECS);
                tracing::warn!(
                    retry_after_secs,
                    "Telegram getUpdates rate limited (429): {description}"
                );
                tokio::time::sleep(Duration::from_secs(retry_after_secs)).await;
            }
            _ => {
                tracing::warn!("Telegram getUpdates API error (code={error_code}): {description}");
                tokio::time::sleep(Duration::from_secs(TELEGRAM_POLL_RETRY_SECS)).await;
            }
        }

        Ok(PollUpdatesOutcome::Continue)
    }

    async fn process_update_batch(
        &self,
        tx: &mpsc::Sender<ChannelMessage>,
        data: &Value,
        offset: &mut i64,
    ) -> anyhow::Result<bool> {
        if let Some(results) = data.get("result").and_then(Value::as_array) {
            for update in results {
                if let Some(uid) = update.get("update_id").and_then(Value::as_i64) {
                    *offset = uid + 1;
                }
                let Some(mut msg) = self.parse_update_message(update) else {
                    continue;
                };
                self.enrich_inbound_message_with_media_attachments(update, &mut msg)
                    .await;
                let _ = self.send_chat_action(&msg.recipient, "typing").await;
                if tx.send(msg).await.is_err() {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }
}
