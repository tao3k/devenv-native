//! Public data contracts for semantic `SSOT` artifacts and bundles.

mod model;

pub use self::model::{
    SemanticBundleProvenance, SemanticChangeIntent, SemanticChangeIntentType, SemanticConfidence,
    SemanticConfidenceSource, SemanticObject, SemanticObjectKind, SemanticOwner,
    SemanticProjection, SemanticProjectionFreshnessPolicyEntry,
    SemanticProjectionFreshnessPolicyReport, SemanticProjectionPolicyStatus,
    SemanticProjectionRefreshPlanEntry, SemanticProjectionRefreshPlanReport,
    SemanticProjectionRefreshPlanStatus, SemanticProjectionStaleness, SemanticProjectionType,
    SemanticProvenance, SemanticRelation, SemanticRelationChange, SemanticRelationChangeAction,
    SemanticRelationEdge, SemanticRelationKind, SemanticRepository, SemanticScopeBundle,
    SemanticScopeMetadataEnvelope, SemanticScopeRequest, SemanticStatus, SemanticStatusTransition,
    SemanticValidationIssue, SemanticValidationReport, SemanticVerification,
};
