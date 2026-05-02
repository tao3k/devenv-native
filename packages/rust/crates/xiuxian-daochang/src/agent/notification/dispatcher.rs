//! Notification dispatch loop for agent state change events.

use std::sync::Arc;

use anyhow::Result;
use tokio::sync::RwLock;

use super::NotificationProvider;

/// Dispatch result metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotificationReceipt {
    /// Provider that delivered the message.
    pub provider: &'static str,
    /// Original recipient selector.
    pub recipient: String,
}

/// Runtime plugin registry for outbound notifications.
#[derive(Default)]
pub struct NotificationDispatcher {
    providers: RwLock<Vec<Arc<dyn NotificationProvider>>>,
}

impl NotificationDispatcher {
    /// Create an empty dispatcher registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register provider instance in append order.
    pub async fn register(&self, provider: Arc<dyn NotificationProvider>) {
        let mut providers = self.providers.write().await;
        providers.push(provider);
    }

    /// Dispatch one notification through matching providers.
    ///
    /// Providers are tried in registration order.
    ///
    /// # Errors
    /// Returns an error when no provider supports the recipient selector or all matched providers fail.
    pub async fn dispatch(&self, recipient: &str, content: &str) -> Result<NotificationReceipt> {
        let providers = self.providers.read().await.clone();
        let mut attempted = false;
        let mut errors = Vec::new();

        for provider in providers {
            if !provider.supports(recipient) {
                continue;
            }
            attempted = true;
            match provider.send(recipient, content).await {
                Ok(()) => {
                    return Ok(NotificationReceipt {
                        provider: provider.name(),
                        recipient: recipient.to_string(),
                    });
                }
                Err(error) => {
                    tracing::warn!(
                        event = "notification.dispatch.provider_failed",
                        provider = provider.name(),
                        recipient,
                        error = %error,
                        "notification provider dispatch failed; trying next provider"
                    );
                    errors.push(format!("{}: {error}", provider.name()));
                }
            }
        }

        if !attempted {
            anyhow::bail!("no notification provider matched recipient `{recipient}`");
        }
        anyhow::bail!(
            "all matching notification providers failed for `{recipient}`: {}",
            errors.join("; ")
        );
    }
}
