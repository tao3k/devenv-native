//! Knowledge Graph — high-performance Rust implementation.
//!
//! Provides Entity and Relation types for knowledge graph operations.

/// Core knowledge graph implementation.
#[path = "graph/core.rs"]
pub mod core;
#[path = "graph/dedup/mod.rs"]
mod dedup;
#[path = "graph/entity_ops.rs"]
mod entity_ops;
#[path = "graph/errors.rs"]
mod errors;
#[path = "graph/intent/mod.rs"]
mod intent;
#[path = "graph/persistence/mod.rs"]
mod persistence;
#[path = "graph/query/mod.rs"]
pub mod query;
#[path = "graph/relation_ops.rs"]
mod relation_ops;
#[path = "graph/skill_registry.rs"]
mod skill_registry;
#[path = "graph/stats.rs"]
mod stats;
#[path = "graph/valkey_persistence.rs"]
mod valkey_persistence;

pub use crate::entity::{
    Entity, EntitySearchQuery, EntityType, GraphStats, MultiHopOptions, Relation, RelationType,
};
pub use core::{KnowledgeGraph, read_lock, write_lock};
pub use errors::GraphError;
pub use intent::{QueryIntent, extract_intent};
pub use skill_registry::{SkillDoc, SkillRegistrationResult};
