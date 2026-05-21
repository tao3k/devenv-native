//! # xiuxian-types
//!
//! Shared foundational types for the Xiuxian runtime.

mod types;

pub use types::{
    AgentContext, AgentResult, EnvironmentSnapshot, KnowledgeCategory, MemoryGateDecision,
    MemoryGateVerdict, MemoryPromotionTarget, OmegaDecision, OmegaFallbackPolicy, OmegaRiskLevel,
    OmegaRoute, OmegaToolTrustClass, OmniError, OmniResult, SchemaError, Skill, SkillDefinition,
    TaskBrief, VectorSearchResult, get_registered_types, get_schema_json,
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
