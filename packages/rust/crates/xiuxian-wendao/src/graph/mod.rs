//! Knowledge Graph — high-performance Rust implementation.
//!
//! Provides Entity and Relation types for knowledge graph operations.

/// Core knowledge graph implementation.
#[path = "core.rs"]
pub mod core;
#[path = "dedup/mod.rs"]
mod dedup;
#[path = "entity_ops.rs"]
mod entity_ops;
#[path = "errors.rs"]
mod errors;
#[path = "intent/mod.rs"]
mod intent;
#[path = "persistence/mod.rs"]
mod persistence;
#[path = "query/mod.rs"]
pub mod query;
#[path = "relation_ops.rs"]
mod relation_ops;
#[path = "skill_registry.rs"]
mod skill_registry;
#[path = "stats.rs"]
mod stats;
#[path = "valkey_persistence.rs"]
mod valkey_persistence;

pub use crate::entity::{
    Entity, EntitySearchQuery, EntityType, GraphStats, MultiHopOptions, Relation, RelationType,
};
pub use core::{KnowledgeGraph, read_lock, write_lock};
pub use errors::GraphError;
pub use intent::{QueryIntent, extract_intent};
pub use skill_registry::{SkillDoc, SkillRegistrationResult};
