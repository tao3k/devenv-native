//! `parsers::search` owns Wendao parsers search behavior.

/// Repo-search query parsing.
#[cfg(feature = "search-runtime")]
#[path = "repo_code_query/mod.rs"]
pub mod repo_code_query;
