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
