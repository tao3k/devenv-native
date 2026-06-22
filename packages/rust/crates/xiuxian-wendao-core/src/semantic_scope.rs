//! Stable semantic-scope metadata DTOs shared with runtime consumers.
//!
//! Parser crates may produce these records, but consumers such as Qianji should
//! depend on this lightweight contract rather than parser or AST crates.

use serde::{Deserialize, Serialize};

macro_rules! semantic_token {
    ($name:ident, $doc:literal) => {
        #[doc = $doc]
        #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            /// Build a semantic token.
            #[must_use]
            pub fn new(value: impl Into<String>) -> Self {
                Self(value.into())
            }

            /// Borrow this token as a string slice.
            #[must_use]
            pub fn as_str(&self) -> &str {
                self.0.as_str()
            }
        }

        impl From<String> for $name {
            fn from(value: String) -> Self {
                Self::new(value)
            }
        }

        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                Self::new(value)
            }
        }
    };
}

semantic_token!(SemanticScopeObjectKind, "Semantic-scope object kind token.");
semantic_token!(
    SemanticScopeRelationKind,
    "Semantic-scope relation kind token."
);
semantic_token!(
    SemanticScopeStatus,
    "Semantic-scope lifecycle status token."
);
semantic_token!(
    SemanticScopeProjectionStaleness,
    "Semantic-scope projection staleness token."
);
semantic_token!(
    SemanticScopeProjectionPolicyStatus,
    "Semantic-scope projection policy status token."
);

/// App metadata key carrying the full semantic-scope bundle.
pub const SEMANTIC_SCOPE_BUNDLE_METADATA_KEY: &str = "semanticScopeBundle";

/// App metadata key carrying semantic projection freshness policy evidence.
pub const SEMANTIC_PROJECTION_POLICY_EVIDENCE_METADATA_KEY: &str =
    "semanticProjectionPolicyEvidence";

/// App metadata key carrying semantic SQL guard evidence.
pub const SEMANTIC_SQL_GUARD_EVIDENCE_METADATA_KEY: &str = "semanticSqlGuardEvidence";

/// Semantic-scope metadata envelope shared by producers and consumers.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SemanticScopeMetadataEnvelope {
    /// Full semantic-scope bundle.
    #[serde(rename = "semanticScopeBundle")]
    pub bundle: SemanticScopeBundle,
    /// Optional SQL guard evidence JSON owned by the semantic read-model layer.
    #[serde(rename = "semanticSqlGuardEvidence")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sql_guard_evidence: Option<serde_json::Value>,
    /// Optional projection freshness policy evidence.
    #[serde(rename = "semanticProjectionPolicyEvidence")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub projection_policy_evidence: Option<SemanticProjectionFreshnessPolicyReport>,
}

/// Deterministic semantic-scope bundle returned to runtime consumers.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SemanticScopeBundle {
    /// Optional task anchor ID.
    #[serde(default)]
    pub task_id: Option<String>,
    /// Object IDs requested explicitly by the caller.
    #[serde(default)]
    pub requested_object_ids: Vec<String>,
    /// Included semantic objects.
    #[serde(default)]
    pub objects: Vec<SemanticScopeObject>,
    /// Included relation edges.
    #[serde(default)]
    pub relations: Vec<SemanticScopeRelationEdge>,
    /// Included semantic change intents related to this scope.
    #[serde(default)]
    pub change_intents: Vec<SemanticScopeChangeIntent>,
    /// Included invariant object IDs.
    #[serde(default)]
    pub affected_invariants: Vec<String>,
    /// Deduplicated validation requirements from included objects.
    #[serde(default)]
    pub required_validations: Vec<String>,
    /// Projection revision that best represents this bundle.
    #[serde(default)]
    pub projection_revision: String,
    /// Source revision for the selected projection, when one exists.
    #[serde(default)]
    pub projection_source_revision: Option<String>,
    /// Freshness state for the selected projection, when one exists.
    #[serde(default)]
    pub projection_staleness: Option<SemanticScopeProjectionStaleness>,
    /// Requested IDs that could not be resolved.
    #[serde(default)]
    pub unresolved_ids: Vec<String>,
}

/// Included semantic object summary.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticScopeObject {
    /// Stable semantic object ID.
    pub id: String,
    /// Object kind token.
    pub kind: SemanticScopeObjectKind,
    /// Human-readable object title.
    pub title: String,
    /// Lifecycle status token.
    pub status: SemanticScopeStatus,
}

/// Fully qualified relation edge in a scope bundle.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticScopeRelationEdge {
    /// Source semantic object ID.
    pub source: String,
    /// Relation kind token.
    pub kind: SemanticScopeRelationKind,
    /// Target semantic object ID.
    pub target: String,
}

/// Included semantic change intent summary.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticScopeChangeIntent {
    /// Stable semantic change identifier.
    pub id: String,
}

/// Projection freshness policy report shared by semantic producers and clients.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SemanticProjectionFreshnessPolicyReport {
    /// Stable projection policy identifier.
    pub policy_id: String,
    /// Policy status token.
    pub status: SemanticScopeProjectionPolicyStatus,
    /// Count of projections that require review.
    pub failing_projection_count: usize,
    /// Human-readable policy message.
    pub message: String,
    /// Per-projection policy findings.
    #[serde(default)]
    pub projections: Vec<SemanticProjectionFreshnessPolicyEntry>,
}

/// Per-projection finding for the semantic projection freshness policy.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SemanticProjectionFreshnessPolicyEntry {
    /// Projection name.
    pub projection: String,
    /// Source revision declared by the projection artifact.
    pub source_revision: String,
    /// Source revision computed from current source objects, when resolvable.
    pub current_source_revision: Option<String>,
    /// Projection staleness token.
    pub staleness: String,
    /// Policy failure reason token.
    pub reason: String,
    /// Projection source path relative to the semantic root, when available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_path: Option<String>,
}

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
