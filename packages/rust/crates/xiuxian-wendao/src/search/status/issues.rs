//! `search::status::issues` owns Wendao search status issues behavior.

use serde::{Deserialize, Serialize};

/// Machine-readable issue code attached to a corpus status row.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SearchCorpusIssueCode {
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SearchCorpusIssueFamily {
    /// Issues around missing or malformed published state.
    Manifest,
    /// Issues where the published revision no longer matches the repo revision.
    Revision,
    /// Issues coming from repo indexing/sync failures.
    RepoSync,
    /// Multiple issue families are present at once.
    Mixed,
}

/// Machine-readable issue attached to a corpus status row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchCorpusIssue {
    /// Stable issue code suitable for client-side branching.
    pub code: SearchCorpusIssueCode,
    /// Whether the corpus remains readable despite this issue.
    pub readable: bool,
    /// Repository identifier associated with the issue, when applicable.
    pub repo_id: Option<String>,
    /// Current repo revision observed during status synthesis.
    pub current_revision: Option<String>,
    /// Published revision currently attached to the serving table, when known.
    pub published_revision: Option<String>,
    /// Human-readable message preserved for logs and current UI surfaces.
    pub message: String,
}

/// High-level summary derived from the corpus issue list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchCorpusIssueSummary {
    /// Dominant family for the current issue set.
    pub family: SearchCorpusIssueFamily,
    /// Highest-priority issue code in the current issue set.
    pub primary_code: SearchCorpusIssueCode,
    /// Total number of issues attached to the corpus.
    pub issue_count: usize,
    /// Number of issues that still allow reads to continue.
    pub readable_issue_count: usize,
}
