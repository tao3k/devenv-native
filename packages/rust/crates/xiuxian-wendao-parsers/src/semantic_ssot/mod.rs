//! Parser-owned repo-native semantic SSOT artifact contracts.

mod api;
mod types;

pub use self::api::{
    load_semantic_repository, parse_semantic_change_intent, parse_semantic_object,
    parse_semantic_projection, semantic_projection_source_revision, semantic_scope_bundle,
};
pub use self::types::{
    SemanticBundleProvenance, SemanticChangeIntent, SemanticConfidence, SemanticConfidenceSource,
    SemanticObject, SemanticObjectKind, SemanticOwner, SemanticProjection,
    SemanticProjectionStaleness, SemanticProvenance, SemanticRelation, SemanticRelationChange,
    SemanticRelationChangeAction, SemanticRelationEdge, SemanticRelationKind, SemanticRepository,
    SemanticScopeBundle, SemanticScopeRequest, SemanticStatus, SemanticValidationIssue,
    SemanticValidationReport, SemanticVerification,
};
