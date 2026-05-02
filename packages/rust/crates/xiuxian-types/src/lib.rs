//! # xiuxian-types
//!
//! Shared foundational types for the Xiuxian runtime.

mod types;

pub use types::{
    AgentContext, AgentResult, EnvironmentSnapshot, KnowledgeCategory, MemoryGateDecision,
    MemoryGateVerdict, MemoryPromotionTarget, OmniError, OmniResult, SchemaError, Skill,
    SkillDefinition, TaskBrief, VectorSearchResult, get_registered_types, get_schema_json,
};

#[cfg(test)]
rust_lang_project_harness::rust_project_harness_cargo_test_gate!();
