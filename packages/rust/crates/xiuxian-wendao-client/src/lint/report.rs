//! Serializable report model for markdown lint diagnostics.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// One rendered Markdown lint issue scoped to a specific file.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct MarkdownLintIssue {
    /// File-relative machine-readable lint code.
    pub code: String,
    /// High-level classification for this issue.
    pub kind: String,
    /// Human-readable problem summary.
    pub problem: String,
    /// Human-readable diagnostic detail or fix guidance.
    pub message: String,
    /// One-based source line.
    pub line: usize,
    /// One-based source column.
    pub column: usize,
    /// Target extracted from the offending link when available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    /// Resolved title for the target document when available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_title: Option<String>,
    /// Heading fragment extracted from the target when available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_heading: Option<String>,
    /// Exact offending source literal when available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub found: Option<String>,
    /// Concrete expected rewrite when available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected: Option<String>,
    /// Exact source line when available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// Short extra fix hint for human or LLM consumers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tip: Option<String>,
}

/// Aggregate lint results for one Markdown file.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct MarkdownLintFileReport {
    /// Root-relative file path.
    pub path: String,
    /// Stable issue count for this file.
    pub issue_count: usize,
    /// Stable file-local issue list.
    pub issues: Vec<MarkdownLintIssue>,
}

/// Aggregate lint results for one command execution.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct MarkdownLintReport {
    /// Number of Markdown files checked.
    pub checked_files: usize,
    /// Number of files containing at least one issue.
    pub files_with_issues: usize,
    /// Total issue count across all checked files.
    pub issue_count: usize,
    /// Stable file reports in path order.
    pub files: Vec<MarkdownLintFileReport>,
}
