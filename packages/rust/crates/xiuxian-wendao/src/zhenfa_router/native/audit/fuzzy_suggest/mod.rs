//! Fuzzy Pattern Suggestion for Code Observations (Blueprint v2.9).
//!
//! When an `:OBSERVE:` pattern fails validation (e.g., a symbol was renamed),
//! this module searches for similar patterns and suggests updated patterns.
//!
//! ## Architecture
//!
//! 1. **Detect**: Pattern validation fails -> Error diagnostic
//! 2. **Search**: Fuzzy structural search -> Candidate matches
//! 3. **Suggest**: Rank & format suggestions -> `replacement_drawer` content
//!
//! ## Example
//!
//! ```ignore
//! use crate::zhenfa_router::native::audit::fuzzy_suggest::{
//!     suggest_pattern_fix, SourceFile,
//! };
//!
//! let source = SourceFile {
//!     path: "src/lib.rs".to_string(),
//!     content: "fn process_records(data: Vec<u8>) -> Result<()>".to_string(),
//! };
//!
//! let suggestion = suggest_pattern_fix(
//!     "fn process_data($$$)",
//!     &xiuxian_code_intelligence::CodeLanguageId::from("rust"),
//!     &[source],
//! );
//!
//! assert!(suggestion.is_some());
//! assert!(suggestion.unwrap().suggested_pattern.contains("process_records"));
//! ```
//!
//! ## Performance Caching (Blueprint v2.9)
//!
//! To avoid re-scanning the same source files repeatedly, this module
//! uses a thread-local cache for candidate matches. The cache is invalidated
//! when source files change.

mod cache;
mod format;
mod pattern;
mod search;
mod similarity;
mod sources;
mod types;

pub use cache::{cache_stats, clear_candidate_cache};
pub use format::format_suggestion;
pub use search::{suggest_pattern_fix, suggest_pattern_fix_with_threshold};
pub use sources::resolve_source_files;
pub use types::{FuzzySuggestion, SourceFile};

#[cfg(test)]
pub(crate) use cache::CONFIDENCE_THRESHOLD;
#[cfg(test)]
pub(crate) use pattern::{PatternSkeleton, extract_capture_name, tokenize_pattern};
#[cfg(test)]
pub(crate) use similarity::{jaccard_similarity, levenshtein_distance, string_similarity};

#[cfg(test)]
#[path = "../../../../../tests/unit/zhenfa_router/native/audit/fuzzy_suggest.rs"]
mod tests;
