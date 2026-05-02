//! Notification provider trait and in-process provider adapters.

use anyhow::Result;
use async_trait::async_trait;

/// Outbound notification provider abstraction.
#[async_trait]
pub trait NotificationProvider: Send + Sync {
    /// Provider name for diagnostics.
    fn name(&self) -> &'static str;

    /// Whether the provider can handle this recipient selector.
    fn supports(&self, recipient: &str) -> bool;

    /// Send outbound notification content to recipient selector.
    async fn send(&self, recipient: &str, content: &str) -> Result<()>;
}
