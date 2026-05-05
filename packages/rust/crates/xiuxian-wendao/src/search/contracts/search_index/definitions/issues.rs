use serde::{Deserialize, Serialize};
use specta::Type;

/// Machine-readable issue code attached to one search corpus status row.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum SearchIndexIssueCode {
    /// A repo reported ready but no published state exists for this corpus.
    PublishedManifestMissing,
    /// Published state exists, but it does not record the source revision.
    PublishedRevisionMissing,
    /// Published state exists, but it points at a different source revision.
    PublishedRevisionMismatch,
    /// Repo indexing failed while the corpus status was synthesized.
    RepoIndexFailed,
}

/// High-level issue family used to summarize corpus status for UI consumers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum SearchIndexIssueFamily {
    /// Issues around missing or malformed published state.
    Manifest,
    /// Issues where the published revision no longer matches the repo revision.
    Revision,
    /// Issues coming from repo indexing/sync failures.
    RepoSync,
    /// Multiple issue families are present at once.
    Mixed,
}

/// Machine-readable issue attached to one search corpus status row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SearchIndexIssue {
    /// Stable issue code suitable for client-side branching.
    pub code: SearchIndexIssueCode,
    /// Whether the corpus remains readable despite this issue.
    pub readable: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Repository identifier associated with the issue, when applicable.
    pub repo_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Current repo revision observed during status synthesis.
    pub current_revision: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Published revision currently attached to the serving table, when known.
    pub published_revision: Option<String>,
    /// Human-readable message preserved for current UI surfaces.
    pub message: String,
}

/// High-level summary derived from the issue list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SearchIndexIssueSummary {
    /// Dominant family for the current issue set.
    pub family: SearchIndexIssueFamily,
    /// Highest-priority issue code in the current issue set.
    pub primary_code: SearchIndexIssueCode,
    /// Total number of issues attached to the corpus.
    pub issue_count: usize,
    /// Number of issues that still allow reads to continue.
    pub readable_issue_count: usize,
}
