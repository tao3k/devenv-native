//! Repo-native semantic SSOT loading, validation, and scope APIs.

mod artifact;
mod hash;
mod metadata;
mod projection;
mod repository;
mod scope;
mod validate;

pub use self::artifact::{
    SemanticArtifactParseError, parse_semantic_change_intent, parse_semantic_object,
    parse_semantic_projection,
};
pub use self::metadata::{
    SEMANTIC_PROJECTION_POLICY_EVIDENCE_METADATA_KEY, SEMANTIC_SCOPE_BUNDLE_METADATA_KEY,
    SEMANTIC_SQL_GUARD_EVIDENCE_METADATA_KEY, parse_semantic_scope_metadata_envelope_json,
    semantic_scope_metadata_envelope, semantic_scope_metadata_envelope_to_vec,
};
pub use self::projection::{
    SEMANTIC_PROJECTION_FRESHNESS_POLICY_ID, semantic_projection_freshness_policy_report,
    semantic_projection_refresh_plan_report, semantic_projection_source_revision,
};
pub use self::repository::load_semantic_repository;
pub use self::scope::semantic_scope_bundle;
