//! `zhenfa_router::native::forwarder::payload` owns Wendao native forwarder payload behavior.

use serde::{Deserialize, Serialize};

/// A notification payload to be sent to subscribers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForwardNotification {
    /// Unique notification ID.
    pub id: String,
    /// Source file that changed.
    pub source_path: String,
    /// Timestamp of the notification.
    pub timestamp: String,
    /// Affected documents.
    pub affected_docs: Vec<AffectedDocInfo>,
    /// Confidence level.
    pub confidence: String,
    /// Summary of the drift.
    pub summary: String,
    /// Suggested action.
    pub suggested_action: SuggestedAction,
    /// Whether auto-fix is available.
    pub auto_fix_available: bool,
    /// Diff preview (unified diff format) showing what changed.
    /// This helps document maintainers quickly understand the change.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub diff_preview: Option<DiffPreview>,
}

/// A preview of the diff between old and new content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffPreview {
    /// Number of lines added.
    pub lines_added: usize,
    /// Number of lines removed.
    pub lines_removed: usize,
    /// Unified diff snippet (truncated to reasonable size).
    pub unified_diff: String,
    /// Key symbols that were added.
    pub symbols_added: Vec<String>,
    /// Key symbols that were removed.
    pub symbols_removed: Vec<String>,
    /// Maximum diff context lines.
    pub context_lines: usize,
}

/// Information about an affected document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AffectedDocInfo {
    /// Document ID.
    pub doc_id: String,
    /// Matching pattern.
    pub pattern: String,
    /// Language of the observation.
    pub language: String,
    /// Line number if available.
    pub line_number: Option<usize>,
    /// Document owner if known.
    pub owner: Option<String>,
}

/// Suggested action for the notification recipient.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SuggestedAction {
    /// Review the documentation manually.
    Review,
    /// Update the :OBSERVE: pattern to match new code.
    UpdatePattern,
    /// Remove the stale observation.
    RemoveObservation,
    /// Auto-fix is available and can be applied.
    AutoFix,
    /// No action needed (informational only).
    NoAction,
}
