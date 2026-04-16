//! Narrow bounded SQL helper surfaces for downstream Wendao consumers.
//!
//! Ownership rule:
//! - keep stable SQL payload DTOs in `xiuxian-wendao-core`
//! - keep broader shared-query semantics in `xiuxian-wendao`
//! - put small bounded downstream SQL helpers here when they should not drag
//!   the full Wendao feature graph into their callers

/// Bounded-work markdown SQL helper surface for workdir-local retrieval.
pub mod bounded_work_markdown;
/// Request-scoped local relation-engine seams for bounded SQL helpers.
pub mod local_relation;
mod payload;

pub use local_relation::{
    DataFusionLocalRelationEngine, LocalRelationEngine, LocalRelationEngineKind,
    LocalRelationMaterializationState, LocalRelationRegistrationHint,
};
pub use xiuxian_wendao_core::{
    SqlBatchPayload, SqlColumnPayload, SqlQueryMetadata, SqlQueryPayload,
};

xiuxian_testing::crate_test_policy_source_harness!("../tests/unit/lib_policy.rs");
