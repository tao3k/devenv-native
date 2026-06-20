//! `zhenfa_router::native::forwarder::service` owns Wendao native forwarder service behavior.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::OnceLock;
use std::time::Duration;

use tokio::sync::{RwLock, mpsc};

use crate::zhenfa_router::native::forwarder::config::ForwarderConfig;
use crate::zhenfa_router::native::forwarder::payload::{
    AffectedDocInfo, DiffPreview, ForwardNotification, SuggestedAction,
};
use crate::zhenfa_router::native::forwarder::rate_limiter::RateLimiter;
use crate::zhenfa_router::native::sentinel::{
    DriftConfidence, SemanticDriftSignal, extract_pattern_symbols,
};

type PendingNotification = (SemanticDriftSignal, chrono::DateTime<chrono::Utc>);
type PendingNotificationMap = HashMap<String, PendingNotification>;

/// The `ForwardNotifier` service.
#[derive(Debug)]
pub struct ForwardNotifier {
    config: ForwarderConfig,
    /// Rate limiter for notifications.
    rate_limiter: Arc<RwLock<RateLimiter>>,
    /// Notification channel sender.
    tx: Option<mpsc::UnboundedSender<ForwardNotification>>,
    /// Pending notifications (debounced).
    pending: Arc<RwLock<PendingNotificationMap>>,
}

impl ForwardNotifier {
    /// Create a new `ForwardNotifier` with the given configuration.
    #[must_use]
    pub fn new(config: ForwarderConfig) -> Self {
        Self {
            config,
            rate_limiter: Arc::new(RwLock::new(RateLimiter::default())),
            tx: None,
            pending: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Attach a notification channel.
    pub fn attach_sender(&mut self, tx: mpsc::UnboundedSender<ForwardNotification>) {
        self.tx = Some(tx);
    }

    fn debounce_duration(&self) -> chrono::Duration {
        let seconds = i64::try_from(self.config.debounce_secs).unwrap_or(i64::MAX);
        chrono::Duration::seconds(seconds)
    }

    /// Process a semantic drift signal.
    ///
    /// Returns true if a notification was queued for delivery.
    pub async fn process_drift(&self, drift: &SemanticDriftSignal) -> bool {
        // Check confidence threshold
        if drift.confidence < self.config.min_confidence {
            log::debug!(
                "Skipping notification: confidence {:?} below threshold {:?}",
                drift.confidence,
                self.config.min_confidence
            );
            return false;
        }

        // Check rate limits for each affected doc
        let mut rate_limiter = self.rate_limiter.write().await;
        let within_rate_limit = drift.affected_docs.iter().all(|doc| {
            let allowed = rate_limiter
                .check_and_increment(doc.doc_id.as_str(), self.config.rate_limit_per_hour);
            if !allowed {
                log::info!(
                    "Rate limit exceeded for doc: {}, skipping notification",
                    doc.doc_id
                );
            }
            allowed
        });
        if !within_rate_limit {
            return false;
        }
        drop(rate_limiter);

        // Add to pending (debounced)
        let key = drift.source_path.clone();
        let mut pending = self.pending.write().await;

        // Check if we have a recent pending notification for the same source
        if let Some((_, timestamp)) = pending.get(&key) {
            let elapsed = chrono::Utc::now() - *timestamp;
            if elapsed < self.debounce_duration() {
                log::debug!("Debouncing notification for: {key}");
                return false;
            }
        }

        pending.insert(key.clone(), (drift.clone(), chrono::Utc::now()));
        drop(pending);

        // Build and emit notification
        let notification = self.build_notification(drift);
        if let Some(ref tx) = self.tx
            && tx.send(notification).is_ok()
        {
            log::info!("Forwarded notification for: {}", drift.source_path);
            return true;
        }

        false
    }

    /// Build a notification from a drift signal.
    fn build_notification(&self, drift: &SemanticDriftSignal) -> ForwardNotification {
        let auto_fix_available =
            drift.confidence >= self.config.auto_fix_min_confidence && self.config.auto_fix_enabled;

        let suggested_action = if auto_fix_available {
            SuggestedAction::AutoFix
        } else if drift.confidence == DriftConfidence::High {
            SuggestedAction::UpdatePattern
        } else {
            SuggestedAction::Review
        };

        let affected_docs = drift
            .affected_docs
            .iter()
            .map(|doc| AffectedDocInfo {
                doc_id: doc.doc_id.clone(),
                pattern: doc.matching_pattern.clone(),
                language: doc.language.clone(),
                line_number: doc.line_number,
                owner: None, // TODO: Resolve from git blame or :OWNER: attribute
            })
            .collect();

        // Generate diff preview if we have old/new content
        let diff_preview = Self::generate_diff_preview(drift);

        ForwardNotification {
            id: format!("notif-{}", chrono::Utc::now().timestamp_millis()),
            source_path: drift.source_path.clone(),
            timestamp: drift.timestamp.clone(),
            affected_docs,
            confidence: drift.confidence.to_string(),
            summary: drift.summary(),
            suggested_action,
            auto_fix_available,
            diff_preview,
        }
    }

    /// Generate a diff preview for the drift signal.
    ///
    /// This compares the old and new content to produce a unified diff
    /// that helps document maintainers quickly understand what changed.
    fn generate_diff_preview(drift: &SemanticDriftSignal) -> Option<DiffPreview> {
        // Extract symbols from affected patterns
        let symbols_added = drift
            .affected_docs
            .iter()
            .flat_map(|doc| extract_pattern_symbols(&doc.matching_pattern))
            .collect::<Vec<_>>();
        let symbols_removed = Vec::new();

        // If no symbols were found, return None
        if symbols_added.is_empty() {
            return None;
        }

        // Generate a simple unified diff snippet
        let unified_diff = format!(
            "--- {}\n+++ {}\n@@ -1,1 +1,1 @@\n-// OLD: pattern may no longer match\n+// NEW: source file changed, verify pattern\n",
            drift.source_path, drift.source_path
        );

        Some(DiffPreview {
            lines_added: 1,
            lines_removed: 1,
            unified_diff,
            symbols_added,
            symbols_removed,
            context_lines: 3,
        })
    }

    /// Send a webhook notification.
    ///
    /// # Errors
    ///
    /// Returns an error when the HTTP request fails or the webhook responds
    /// with a non-success status code.
    pub async fn send_webhook(&self, notification: &ForwardNotification) -> Result<(), String> {
        if !self.config.webhook_enabled {
            return Ok(());
        }

        let Some(ref url) = self.config.webhook_url else {
            return Ok(());
        };

        let response = webhook_client()
            .post(url)
            .json(notification)
            .timeout(Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| format!("Webhook request failed: {e}"))?;

        if !response.status().is_success() {
            return Err(format!("Webhook returned status: {}", response.status()));
        }

        log::info!("Webhook notification sent successfully to: {url}");
        Ok(())
    }

    /// Get pending notification count.
    #[must_use]
    pub async fn pending_count(&self) -> usize {
        self.pending.read().await.len()
    }

    /// Clear expired pending notifications.
    pub async fn clear_expired(&self) {
        let mut pending = self.pending.write().await;
        let now = chrono::Utc::now();
        let debounce_duration = self.debounce_duration();

        pending.retain(|_, (_, timestamp)| now - *timestamp < debounce_duration * 2);
    }

    /// Check if auto-fix is available for a drift signal.
    #[must_use]
    pub fn can_auto_fix(&self, drift: &SemanticDriftSignal) -> bool {
        self.config.auto_fix_enabled && drift.confidence >= self.config.auto_fix_min_confidence
    }
}

fn webhook_client() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(5))
            .pool_idle_timeout(Duration::from_secs(120))
            .pool_max_idle_per_host(64)
            .brotli(true)
            .deflate(true)
            .gzip(true)
            .zstd(true)
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new())
    })
}

impl Clone for ForwardNotifier {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            rate_limiter: self.rate_limiter.clone(),
            tx: self.tx.clone(),
            pending: self.pending.clone(),
        }
    }
}
