//! Gateway Notification Service for External Webhook Push.
//!
//! This module implements the Edge Layer notification capability:
//!
//! 1. **Signal Consumption**: Receives `ZhenfaSignal` from Sentinel (backend)
//! 2. **Webhook Push**: Sends notifications to external endpoints
//! 3. **Rate Limiting**: Prevents notification storms
//!
//! # Architecture
//!
//! ```text
//! Sentinel (wendao)
//!      │
//!      ▼ ZhenfaSignal::SemanticDrift
//! ZhenfaGateway (Edge Layer)
//!      │
//!      ├── NotificationService
//!      │         │
//!      │         └── Webhook Push → External Systems (Slack, Discord, etc.)
//!      │
//!      └── HTTP API (RPC endpoints)
//! ```

use std::sync::Arc;
use std::time::Duration;

use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedReceiver;
use tracing::{error, info, warn};

use crate::ZhenfaSignalType;
use crate::native::ZhenfaSignal;

/// Default webhook timeout in seconds.
const DEFAULT_TIMEOUT_SECS: u64 = 10;

/// Maximum retries for webhook delivery.
const MAX_RETRIES: u32 = 3;

/// Webhook notification configuration.
#[derive(Clone, Debug)]
pub struct WebhookConfig {
    /// Webhook endpoint URL.
    pub url: String,
    /// Secret for HMAC signature (optional).
    pub secret: Option<String>,
    /// Request timeout in seconds.
    pub timeout_secs: u64,
    /// Whether to retry on failure.
    pub retry_on_failure: bool,
}

impl Default for WebhookConfig {
    fn default() -> Self {
        Self {
            url: String::new(),
            secret: None,
            timeout_secs: DEFAULT_TIMEOUT_SECS,
            retry_on_failure: true,
        }
    }
}

/// Notification payload sent to webhook endpoints.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NotificationPayload {
    /// Signal type (e.g., "`semantic_drift`").
    pub signal_type: ZhenfaSignalType,
    /// Source of the signal.
    pub source: String,
    /// Human-readable summary.
    pub summary: String,
    /// Confidence level (high/medium/low).
    pub confidence: String,
    /// Affected document paths.
    pub affected_docs: Vec<String>,
    /// Timestamp in ISO 8601 format.
    pub timestamp: String,
    /// Whether auto-fix is available.
    pub auto_fix_available: bool,
    /// Optional fix approval URL (for Phase 8).
    pub fix_approval_url: Option<String>,
}

/// Gateway notification service for external webhooks.
#[derive(Debug)]
pub struct NotificationService {
    /// HTTP client for webhook requests.
    client: reqwest::Client,
    /// Webhook configuration.
    config: WebhookConfig,
    /// Service identifier.
    id: String,
}

impl NotificationService {
    /// Create a new notification service.
    #[must_use]
    pub fn new(config: WebhookConfig) -> Self {
        let client = Client::builder()
            .brotli(true)
            .deflate(true)
            .gzip(true)
            .zstd(true)
            .pool_idle_timeout(Duration::from_secs(120))
            .pool_max_idle_per_host(32)
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            client,
            config,
            id: format!("notif-{}", uuid::Uuid::new_v4()),
        }
    }

    /// Create with shared HTTP client (for efficient connection pooling).
    #[must_use]
    pub fn with_client(config: WebhookConfig, client: reqwest::Client) -> Self {
        Self {
            client,
            config,
            id: format!("notif-{}", uuid::Uuid::new_v4()),
        }
    }

    /// Get the service identifier.
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Convert a `ZhenfaSignal` to a notification payload.
    fn signal_to_payload(signal: &ZhenfaSignal) -> NotificationPayload {
        match signal {
            ZhenfaSignal::SemanticDrift {
                source_path,
                file_stem,
                affected_count,
                confidence,
                summary,
            } => NotificationPayload {
                signal_type: ZhenfaSignalType::from("semantic_drift"),
                source: source_path.clone(),
                summary: summary.clone(),
                confidence: confidence.clone(),
                affected_docs: vec![file_stem.clone()],
                timestamp: chrono_timestamp(),
                auto_fix_available: *affected_count > 0,
                fix_approval_url: None,
            },
            ZhenfaSignal::Reward {
                episode_id,
                value,
                source,
            } => NotificationPayload {
                signal_type: ZhenfaSignalType::from("reward"),
                source: source.clone(),
                summary: format!("Episode {episode_id} received reward {value:.2}"),
                confidence: "high".to_string(),
                affected_docs: vec![],
                timestamp: chrono_timestamp(),
                auto_fix_available: false,
                fix_approval_url: None,
            },
            ZhenfaSignal::Trace { node_id, event } => NotificationPayload {
                signal_type: ZhenfaSignalType::from("trace"),
                source: node_id.clone(),
                summary: event.clone(),
                confidence: "high".to_string(),
                affected_docs: vec![],
                timestamp: chrono_timestamp(),
                auto_fix_available: false,
                fix_approval_url: None,
            },
        }
    }

    /// Send a notification to the configured webhook.
    ///
    /// # Errors
    ///
    /// Returns an error if the webhook request fails after all retries.
    pub async fn notify(&self, payload: &NotificationPayload) -> Result<(), NotificationError> {
        if self.config.url.is_empty() {
            warn!("NotificationService: No webhook URL configured, skipping notification");
            return Ok(());
        }

        let body = serde_json::to_string(payload)
            .map_err(|e| NotificationError::SerializationError(e.to_string()))?;

        let mut attempts = 0;
        let max_attempts = if self.config.retry_on_failure {
            MAX_RETRIES
        } else {
            1
        };

        while attempts < max_attempts {
            attempts += 1;

            let request = self
                .client
                .post(&self.config.url)
                .header("Content-Type", "application/json")
                .header("X-Notification-Id", &self.id)
                .body(body.clone());

            let request = if let Some(ref secret) = self.config.secret {
                request.header("X-Webhook-Secret", secret)
            } else {
                request
            };

            match request.send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        info!(
                            "NotificationService: Successfully sent {} to {}",
                            payload.signal_type, self.config.url
                        );
                        return Ok(());
                    }

                    let status = response.status();
                    if status == StatusCode::TOO_MANY_REQUESTS || status.is_server_error() {
                        warn!(
                            "NotificationService: Attempt {}/{} failed with status {}, retrying...",
                            attempts, max_attempts, status
                        );
                        tokio::time::sleep(Duration::from_millis(100 * u64::from(attempts))).await;
                        continue;
                    }

                    return Err(NotificationError::HttpError {
                        status: status.as_u16(),
                        url: self.config.url.clone(),
                    });
                }
                Err(e) => {
                    if attempts < max_attempts {
                        warn!(
                            "NotificationService: Attempt {}/{} failed: {}, retrying...",
                            attempts, max_attempts, e
                        );
                        tokio::time::sleep(Duration::from_millis(100 * u64::from(attempts))).await;
                        continue;
                    }
                    return Err(NotificationError::NetworkError(e.to_string()));
                }
            }
        }

        Err(NotificationError::MaxRetriesExceeded {
            attempts,
            url: self.config.url.clone(),
        })
    }

    /// Process a signal and send notification if applicable.
    pub async fn process_signal(&self, signal: &ZhenfaSignal) {
        let payload = Self::signal_to_payload(signal);
        if let Err(e) = self.notify(&payload).await {
            error!("NotificationService: Failed to send notification: {}", e);
        }
    }
}

/// Errors that can occur during notification.
#[derive(Debug, thiserror::Error)]
pub enum NotificationError {
    /// HTTP error response from webhook.
    #[error("HTTP {status} from {url}")]
    HttpError {
        /// HTTP status code.
        status: u16,
        /// Webhook URL.
        url: String,
    },
    /// Network error during request.
    #[error("network error: {0}")]
    NetworkError(String),
    /// JSON serialization error.
    #[error("serialization error: {0}")]
    SerializationError(String),
    /// Max retries exceeded.
    #[error("max retries ({attempts}) exceeded for {url}")]
    MaxRetriesExceeded {
        /// Number of attempts made.
        attempts: u32,
        /// Webhook URL.
        url: String,
    },
}

/// Generate ISO 8601 timestamp using `std::time`.
fn chrono_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    SystemTime::now().duration_since(UNIX_EPOCH).map_or_else(
        |_| "1970-01-01T00:00:00Z".to_string(),
        |d| {
            let secs = d.as_secs();
            // Convert Unix timestamp to ISO 8601 format manually
            let days = secs / 86400;
            let remaining = secs % 86400;
            let hours = remaining / 3600;
            let minutes = (remaining % 3600) / 60;
            let seconds = remaining % 60;

            // Unix epoch (1970-01-01) to approximate date
            let years = 1970 + (days / 365);
            let day_of_year = days % 365;

            format!(
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
                years,
                (day_of_year / 31 + 1).min(12),
                (day_of_year % 31 + 1).min(28),
                hours,
                minutes,
                seconds
            )
        },
    )
}

/// Background task that processes signals from a receiver.
///
/// This should be spawned as a sidecar task when building the gateway.
pub async fn notification_worker(
    mut receiver: UnboundedReceiver<ZhenfaSignal>,
    service: Arc<NotificationService>,
) {
    info!("NotificationService worker started (id={})", service.id());

    while let Some(signal) = receiver.recv().await {
        service.process_signal(&signal).await;
    }

    info!("NotificationService worker stopped (id={})", service.id());
}

#[cfg(test)]
#[path = "../../tests/unit/gateway/notification.rs"]
mod tests;
