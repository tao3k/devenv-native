//! Narrow bounded SQL helper surfaces for downstream Wendao consumers.
//!
//! Ownership rule:
//! - keep stable SQL payload DTOs in `xiuxian-wendao-core`
//! - keep broader shared-query semantics in `xiuxian-wendao`
//! - put small bounded downstream SQL helpers here when they should not drag
//!   the full Wendao feature graph into their callers

/// Arrow schema contracts for bounded SQL data-plane tables.
mod arrow_contract;
/// Bounded-work markdown SQL helper surface for workdir-local retrieval.
pub mod bounded_work_markdown;
/// DuckDB inspection surface for Episteme candidate Parquet read models.
pub mod candidate_read_model;
/// Dataset-to-ontology SQL materialization helper surface.
pub mod dataset_ontology;
/// Request-scoped local relation-engine seams for bounded SQL helpers.
pub mod local_relation;
mod payload;
/// Provisional semantic SSOT read-model tables for bounded SQL evidence.
pub mod semantic_read_model;

pub use local_relation::{
    DuckDbLocalRelationEngine, LocalRelationEngine, LocalRelationEngineKind,
    LocalRelationMaterializationState, LocalRelationRegistrationHint,
};
pub use xiuxian_wendao_core::{
    SqlBatchPayload, SqlColumnPayload, SqlQueryMetadata, SqlQueryPayload,
};

#[cfg(test)]
rust_lang_project_harness::rust_project_harness_cargo_test_gate!(
    config = {
        rust_lang_project_harness::default_rust_harness_config().with_verification_profile_hint(
            rust_lang_project_harness::RustVerificationProfileHint::new(
                "src/lib.rs",
                [rust_lang_project_harness::RustOwnerResponsibility::PublicApi],
            )
            .with_rationale("crate root owns the public package API for cargo-test verification"),
        )
    }
);
