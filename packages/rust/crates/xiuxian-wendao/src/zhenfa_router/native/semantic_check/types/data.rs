//! Data types for semantic checking.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::str::FromStr;

use crate::link_graph::PageIndexNode;
use crate::zhenfa_router::native::audit::FuzzySuggestion;
use crate::zhenfa_router::native::semantic_check::episteme::EpistemeLoadReport;

/// Node status values (Blueprint v2.0 Section 3.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NodeStatus {
    /// Node is stable and can be safely referenced.
    #[default]
    Stable,
    /// Node is a draft, may change without notice.
    Draft,
    /// Node is deprecated, references should be updated.
    Deprecated,
}

impl NodeStatus {
    /// Parse status from string.
    #[must_use]
    pub fn parse_lossy(s: &str) -> Self {
        match s.trim().to_uppercase().as_str() {
            "DRAFT" => Self::Draft,
            "DEPRECATED" => Self::Deprecated,
            _ => Self::Stable,
        }
    }
}

impl FromStr for NodeStatus {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::parse_lossy(s))
    }
}

/// Arguments for semantic check tool.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct WendaoSemanticCheckArgs {
    /// Document stem or ID to check (optional, checks all docs if not specified).
    #[serde(default)]
    pub doc: Option<String>,
    /// Check types to run (default: all).
    #[serde(default)]
    pub checks: Option<Vec<CheckType>>,
    /// Include warnings in addition to errors.
    #[serde(default)]
    pub include_warnings: Option<bool>,
    /// Source file paths to scan for fuzzy pattern suggestions (Blueprint v2.9).
    /// When provided, invalid `:OBSERVE:` patterns will get suggested fixes.
    #[serde(default)]
    pub source_paths: Option<Vec<String>>,
    /// Minimum confidence threshold for fuzzy pattern suggestions (0.0 - 1.0).
    /// Default is 0.65. Lower values will suggest more matches, higher values will be more strict.
    #[serde(default)]
    pub fuzzy_confidence_threshold: Option<f32>,
    /// Optional episteme manifest path loaded by `wendao audit --load`.
    #[serde(default)]
    pub episteme_load: Option<String>,
}

/// Types of semantic checks available.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
pub enum CheckType {
    /// Check for dead links (references to non-existent IDs).
    DeadLinks,
    /// Check for references to deprecated nodes.
    DeprecatedRefs,
    /// Validate :CONTRACT: constraints.
    Contracts,
    /// Check for ID collisions (same ID in multiple locations).
    IdCollisions,
    /// Check hash alignment (`expect_hash` vs actual `content_hash`).
    HashAlignment,
    /// Check for missing mandatory :ID: property drawer (Blueprint v2.2).
    MissingIdentity,
    /// Check for legacy syntax markers (Blueprint v2.2).
    LegacySyntax,
    /// Validate :OBSERVE: code patterns using the language-provider boundary.
    CodeObservations,
    /// Validate package-local crate docs governance rules.
    DocGovernance,
}

/// Result of a semantic check operation.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
/// Stringly state boundary: this public record preserves serialized catalog tokens from external or stored Wendao data.
pub struct SemanticCheckResult {
    /// Overall status: "pass", "warning", or "fail".
    pub status: String,
    /// Total issues found.
    pub issue_count: usize,
    /// List of issues found.
    pub issues: Vec<SemanticIssue>,
    /// Summary message.
    pub summary: String,
    /// Per-document audit reports with health scores.
    pub file_reports: Vec<FileAuditReport>,
    /// Loaded episteme manifest summary, when `--load` is used.
    pub episteme: Option<EpistemeLoadReport>,
}

/// Per-document audit report with health score (Blueprint v2.2).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FileAuditReport {
    /// Document path.
    pub path: String,
    /// Health score (0-100, where 100 is perfect).
    pub health_score: u8,
    /// Number of errors in this document.
    pub error_count: usize,
    /// Number of warnings in this document.
    pub warning_count: usize,
}

/// A single semantic issue found during check.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
/// Stringly state boundary: this public record preserves serialized catalog tokens from external or stored Wendao data.
pub struct SemanticIssue {
    /// Issue severity: "error", "warning", or "info".
    pub severity: String,
    /// Issue type: "`dead_link`", "`deprecated_ref`", "`contract_violation`".
    pub issue_type: String,
    /// Document where the issue was found.
    pub doc: String,
    /// Node ID where the issue was found.
    pub node_id: String,
    /// Human-readable description.
    pub message: String,
    /// Optional location information.
    pub location: Option<IssueLocation>,
    /// Suggested fix (if available).
    pub suggestion: Option<String>,
    /// Structured fuzzy suggestion for code observation patterns (Blueprint v2.9).
    pub fuzzy_suggestion: Option<FuzzySuggestionData>,
}

/// Structured fuzzy suggestion data for XML output (Blueprint v2.9).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FuzzySuggestionData {
    /// The original invalid pattern.
    pub original_pattern: String,
    /// The suggested updated pattern.
    pub suggested_pattern: String,
    /// Similarity score (0.0 - 1.0).
    pub confidence: f32,
    /// Source location where match was found.
    pub source_location: Option<String>,
    /// Ready-to-use replacement drawer content.
    pub replacement_drawer: String,
}

impl FuzzySuggestionData {
    /// Create from a raw fuzzy suggestion and the original pattern.
    pub(in crate::zhenfa_router::native::semantic_check) fn from_suggestion(
        s: FuzzySuggestion,
        original: String,
    ) -> Self {
        Self {
            original_pattern: original,
            suggested_pattern: s.suggested_pattern,
            confidence: s.confidence,
            source_location: s.source_location,
            replacement_drawer: s.replacement_drawer,
        }
    }
}

/// Location information for an issue.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct IssueLocation {
    /// Line number (1-based).
    pub line: usize,
    /// Heading path.
    pub heading_path: String,
    /// Byte range (start, end) for precise AST-level mutations.
    pub byte_range: Option<(usize, usize)>,
}

impl IssueLocation {
    /// Create an `IssueLocation` from a `PageIndexNode`'s metadata.
    pub(in crate::zhenfa_router::native::semantic_check) fn from_node(
        node: &PageIndexNode,
    ) -> Self {
        Self {
            line: node.metadata.line_range.0,
            heading_path: node.metadata.structural_path.join(" / "),
            byte_range: node.metadata.byte_range,
        }
    }
}
