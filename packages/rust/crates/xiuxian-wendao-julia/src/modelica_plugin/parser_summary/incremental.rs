//! Incremental parser-summary helpers for Modelica sources.

use serde::Serialize;
use xiuxian_wendao_core::repo_intelligence::{RegisteredRepository, RepoIntelligenceError};

use crate::modelica_plugin::discovery::modelica_doc_surface_semantic_markers;
use crate::modelica_plugin::types::ModelicaSourceId;

use super::fetch::fetch_modelica_parser_file_summary_blocking_for_repository;
use super::types::ModelicaParserFileSummary;

/// Return a stable semantic fingerprint for one Modelica parser-summary file
/// payload.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when the repository does not declare a
/// usable parser-summary client or when the remote parser-summary request
/// fails.
pub fn modelica_parser_summary_file_semantic_fingerprint_for_repository(
    repository: &RegisteredRepository,
    source_id: ModelicaSourceId<'_>,
    source_text: &str,
) -> Result<String, RepoIntelligenceError> {
    let source_id = source_id.as_str();
    let summary = fetch_modelica_parser_file_summary_blocking_for_repository(
        repository,
        source_id,
        source_text,
    )?;
    Ok(modelica_file_semantic_fingerprint(
        source_id,
        source_text,
        &summary,
    ))
}

#[must_use]
pub(crate) fn modelica_parser_file_summary_semantic_fingerprint(
    summary: &ModelicaParserFileSummary,
) -> String {
    stable_payload_fingerprint("modelica_parser_file_summary", summary)
}

#[must_use]
pub(crate) fn modelica_file_semantic_fingerprint(
    source_id: &str,
    source_text: &str,
    summary: &ModelicaParserFileSummary,
) -> String {
    stable_payload_fingerprint(
        "modelica_analysis_file_summary",
        &ModelicaFileSemanticFingerprint {
            parser_summary_fingerprint: modelica_parser_file_summary_semantic_fingerprint(summary),
            doc_surface_markers: modelica_doc_surface_semantic_markers(source_id, source_text),
        },
    )
}

#[derive(Serialize)]
struct ModelicaFileSemanticFingerprint {
    parser_summary_fingerprint: String,
    doc_surface_markers: Vec<String>,
}

#[must_use]
fn stable_payload_fingerprint<T: Serialize + ?Sized>(kind: &str, value: &T) -> String {
    let payload = serde_json::to_vec(value).unwrap_or_else(|error| {
        panic!("Modelica parser-summary payload should serialize: {error}");
    });
    let mut hasher = blake3::Hasher::new();
    hasher.update(kind.as_bytes());
    hasher.update(&payload);
    hasher.finalize().to_hex().to_string()
}

#[cfg(test)]
#[path = "../../../tests/unit/modelica_plugin/parser_summary_incremental.rs"]
mod tests;
