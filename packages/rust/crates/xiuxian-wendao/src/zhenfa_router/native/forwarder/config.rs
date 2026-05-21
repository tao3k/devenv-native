//! `zhenfa_router::native::forwarder::config` owns Wendao native forwarder config behavior.

use serde::{Deserialize, Serialize};

use crate::zhenfa_router::native::sentinel::DriftConfidence;

/// Configuration for the `ForwardNotifier`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForwarderConfig {
    /// Minimum confidence level to trigger notifications.
    pub min_confidence: DriftConfidence,
    /// Whether to enable webhook notifications.
    pub webhook_enabled: bool,
    /// Webhook URL for notifications.
    pub webhook_url: Option<String>,
    /// Whether to enable auto-fix mode.
    pub auto_fix_enabled: bool,
    /// Minimum confidence for auto-fix (higher than notification threshold).
    pub auto_fix_min_confidence: DriftConfidence,
    /// Debounce duration to avoid notification spam.
    pub debounce_secs: u64,
    /// Maximum notifications per document per hour.
    pub rate_limit_per_hour: usize,
}

impl Default for ForwarderConfig {
    fn default() -> Self {
        Self {
            min_confidence: DriftConfidence::Medium,
            webhook_enabled: false,
            webhook_url: None,
            auto_fix_enabled: false,
            auto_fix_min_confidence: DriftConfidence::High,
            debounce_secs: 60,
            rate_limit_per_hour: 5,
        }
    }
}
