//! Knowledge Graph — high-performance Rust implementation.
//!
//! Provides Entity and Relation types for knowledge graph operations.

/// Core knowledge graph implementation.
pub mod core;
mod dedup;
mod entity_ops;
mod errors;
mod intent;
mod persistence;
pub mod query;
mod relation_ops;
mod skill_registry;
mod stats;
mod valkey_persistence;

pub use crate::entity::{
    Entity, EntitySearchQuery, EntityType, GraphStats, MultiHopOptions, Relation, RelationType,
};
pub use core::{KnowledgeGraph, read_lock, write_lock};
pub use errors::GraphError;
pub use intent::{QueryIntent, extract_intent};
pub use skill_registry::{SkillDoc, SkillRegistrationResult};
