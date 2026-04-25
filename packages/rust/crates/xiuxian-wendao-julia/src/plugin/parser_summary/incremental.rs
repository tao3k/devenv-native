use serde::Serialize;
use xiuxian_wendao_core::repo_intelligence::{RegisteredRepository, RepoIntelligenceError};

use super::fetch::fetch_julia_parser_file_summary_blocking_for_repository;
use super::types::JuliaParserFileSummary;

/// Return whether one Julia source file is safe for leaf-only incremental
/// analysis under the native parser-summary contract.
///
/// Safe incremental files must not declare a module, add imports, or add
/// includes because those changes widen repository structure instead of staying
/// leaf-local.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when the repository does not declare a
/// usable parser-summary client or when the remote parser-summary request
/// fails.
pub fn julia_parser_summary_allows_safe_incremental_file_for_repository(
    repository: &RegisteredRepository,
    source_id: &str,
    source_text: &str,
) -> Result<bool, RepoIntelligenceError> {
    let summary = fetch_julia_parser_file_summary_blocking_for_repository(
        repository,
        source_id,
        source_text,
    )?;
    Ok(summary.module_name.is_none() && summary.imports.is_empty() && summary.includes.is_empty())
}

/// Return a stable semantic fingerprint for one Julia parser-summary file
/// payload.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when the repository does not declare a
/// usable parser-summary client or when the remote parser-summary request
/// fails.
pub fn julia_parser_summary_file_semantic_fingerprint_for_repository(
    repository: &RegisteredRepository,
    source_id: &str,
    source_text: &str,
) -> Result<String, RepoIntelligenceError> {
    let summary = fetch_julia_parser_file_summary_blocking_for_repository(
        repository,
        source_id,
        source_text,
    )?;
    Ok(julia_parser_file_summary_semantic_fingerprint(&summary))
}

#[must_use]
pub(crate) fn julia_parser_file_summary_semantic_fingerprint(
    summary: &JuliaParserFileSummary,
) -> String {
    let mut normalized = summary.clone();
    for symbol in &mut normalized.symbols {
        symbol.line_start = None;
        symbol.line_end = None;
    }
    for docstring in &mut normalized.docstrings {
        docstring.target_line_start = None;
        docstring.target_line_end = None;
    }
    stable_payload_fingerprint("julia_parser_file_summary", &normalized)
}

#[must_use]
fn stable_payload_fingerprint<T: Serialize + ?Sized>(kind: &str, value: &T) -> String {
    let payload = serde_json::to_vec(value).unwrap_or_else(|error| {
        panic!("Julia parser-summary payload should serialize: {error}");
    });
    let mut hasher = blake3::Hasher::new();
    hasher.update(kind.as_bytes());
    hasher.update(&payload);
    hasher.finalize().to_hex().to_string()
}

#[cfg(test)]
#[path = "../../../tests/unit/plugin/parser_summary/incremental.rs"]
mod tests;
