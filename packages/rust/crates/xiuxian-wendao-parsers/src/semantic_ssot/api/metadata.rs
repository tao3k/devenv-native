//! JSON metadata envelope helpers for semantic scope bundles.

use crate::semantic_ssot::types::{
    SemanticProjectionFreshnessPolicyReport, SemanticScopeBundle, SemanticScopeMetadataEnvelope,
};

/// App metadata key carrying the full semantic-scope bundle.
pub const SEMANTIC_SCOPE_BUNDLE_METADATA_KEY: &str = "semanticScopeBundle";

/// App metadata key carrying semantic projection freshness policy evidence.
pub const SEMANTIC_PROJECTION_POLICY_EVIDENCE_METADATA_KEY: &str =
    "semanticProjectionPolicyEvidence";

/// App metadata key carrying semantic SQL guard evidence.
pub const SEMANTIC_SQL_GUARD_EVIDENCE_METADATA_KEY: &str = "semanticSqlGuardEvidence";

/// Build a semantic-scope app metadata envelope.
#[must_use]
pub fn semantic_scope_metadata_envelope(
    semantic_scope_bundle: SemanticScopeBundle,
    semantic_sql_guard_evidence: Option<serde_json::Value>,
    semantic_projection_policy_evidence: Option<SemanticProjectionFreshnessPolicyReport>,
) -> SemanticScopeMetadataEnvelope {
    SemanticScopeMetadataEnvelope {
        bundle: semantic_scope_bundle,
        sql_guard_evidence: semantic_sql_guard_evidence,
        projection_policy_evidence: semantic_projection_policy_evidence,
    }
}

/// Encode one semantic-scope app metadata envelope as JSON bytes.
///
/// # Errors
///
/// Returns [`serde_json::Error`] when the envelope cannot be serialized.
pub fn semantic_scope_metadata_envelope_to_vec(
    envelope: &SemanticScopeMetadataEnvelope,
) -> Result<Vec<u8>, serde_json::Error> {
    serde_json::to_vec(envelope)
}

/// Decode semantic-scope app metadata JSON.
///
/// Accepts either the canonical metadata envelope or a raw
/// [`SemanticScopeBundle`] JSON object for compatibility with early runtime
/// consumers.
///
/// # Errors
///
/// Returns [`serde_json::Error`] when the JSON cannot be parsed or decoded as
/// either supported semantic-scope metadata shape.
pub fn parse_semantic_scope_metadata_envelope_json(
    raw_metadata_json: &str,
) -> Result<SemanticScopeMetadataEnvelope, serde_json::Error> {
    let value = serde_json::from_str::<serde_json::Value>(raw_metadata_json)?;
    semantic_scope_metadata_envelope_from_value(value)
}

fn semantic_scope_metadata_envelope_from_value(
    value: serde_json::Value,
) -> Result<SemanticScopeMetadataEnvelope, serde_json::Error> {
    if value.get(SEMANTIC_SCOPE_BUNDLE_METADATA_KEY).is_some() {
        serde_json::from_value(value)
    } else {
        let semantic_scope_bundle = serde_json::from_value::<SemanticScopeBundle>(value)?;
        Ok(semantic_scope_metadata_envelope(
            semantic_scope_bundle,
            None,
            None,
        ))
    }
}
