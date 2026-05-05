//! Parser-owned repo-native semantic SSOT artifact contracts.

mod api;
mod types;

pub use self::api::{
    SEMANTIC_PROJECTION_FRESHNESS_POLICY_ID, SEMANTIC_PROJECTION_POLICY_EVIDENCE_METADATA_KEY,
    SEMANTIC_SCOPE_BUNDLE_METADATA_KEY, SEMANTIC_SQL_GUARD_EVIDENCE_METADATA_KEY,
    load_semantic_repository, parse_semantic_change_intent, parse_semantic_object,
    parse_semantic_projection, parse_semantic_scope_metadata_envelope_json,
    semantic_projection_freshness_policy_report, semantic_projection_refresh_plan_report,
    semantic_projection_source_revision, semantic_scope_bundle, semantic_scope_metadata_envelope,
    semantic_scope_metadata_envelope_to_vec,
};
pub use self::types::{
    SemanticBundleProvenance, SemanticChangeIntent, SemanticConfidence, SemanticConfidenceSource,
    SemanticObject, SemanticObjectKind, SemanticOwner, SemanticProjection,
    SemanticProjectionFreshnessPolicyEntry, SemanticProjectionFreshnessPolicyReport,
    SemanticProjectionRefreshPlanEntry, SemanticProjectionRefreshPlanReport,
    SemanticProjectionStaleness, SemanticProvenance, SemanticRelation, SemanticRelationChange,
    SemanticRelationChangeAction, SemanticRelationEdge, SemanticRelationKind, SemanticRepository,
    SemanticScopeBundle, SemanticScopeMetadataEnvelope, SemanticScopeRequest, SemanticStatus,
    SemanticStatusTransition, SemanticValidationIssue, SemanticValidationReport,
    SemanticVerification,
};
