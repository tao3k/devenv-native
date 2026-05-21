//! `zhenfa_router::native::audit::fuzzy_suggest::types` owns Wendao audit fuzzy suggest types behavior.

use serde::{Deserialize, Serialize};

use super::pattern::PatternSkeleton;

/// A source file to scan for pattern matches.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceFile {
    /// File path (for diagnostics).
    pub path: String,
    /// Source code content.
    pub content: String,
}

/// Result of a fuzzy pattern suggestion search.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuzzySuggestion {
    /// The suggested updated pattern.
    pub suggested_pattern: String,
    /// Similarity score (0.0 - 1.0).
    pub confidence: f32,
    /// Source location where match was found.
    pub source_location: Option<String>,
    /// Ready-to-use replacement drawer content.
    pub replacement_drawer: String,
}

/// A candidate match found in source files.
#[derive(Debug, Clone)]
pub(crate) struct CandidateMatch {
    /// The matched text from source.
    pub(crate) matched_text: String,
    /// Source file path.
    pub(crate) file_path: String,
    /// Line number (1-indexed).
    pub(crate) line_number: usize,
    /// Identifier that was matched (if any).
    pub(crate) identifier: Option<String>,
    /// Skeleton of the matched code.
    pub(crate) skeleton: PatternSkeleton,
}
