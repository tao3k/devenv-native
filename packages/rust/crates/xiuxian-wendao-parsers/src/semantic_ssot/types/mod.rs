//! Public data contracts for semantic `SSOT` artifacts and bundles.

mod model;

pub use self::model::{
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
