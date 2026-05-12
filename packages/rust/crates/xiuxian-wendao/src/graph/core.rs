//! `graph::core` owns Wendao graph core behavior.

use crate::entity::{Entity, Relation};
use std::collections::{HashMap, HashSet};
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

/// Acquire a read lock on a [`RwLock`].
///
/// Returns the inner guard even if the lock is poisoned.
pub fn read_lock<T>(lock: &RwLock<T>) -> RwLockReadGuard<'_, T> {
    lock.read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// Acquire a write lock on a [`RwLock`].
///
/// Returns the inner guard even if the lock is poisoned.
pub fn write_lock<T>(lock: &RwLock<T>) -> RwLockWriteGuard<'_, T> {
    lock.write()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// In-memory knowledge graph state.
#[derive(Debug, Default)]
pub struct KnowledgeGraph {
    pub(crate) entities: RwLock<HashMap<String, Entity>>,
    pub(crate) entities_by_name: RwLock<HashMap<String, String>>,
    pub(crate) entities_by_type: RwLock<HashMap<String, Vec<String>>>,
    pub(crate) relations: RwLock<HashMap<String, Relation>>,
    pub(crate) outgoing_relations: RwLock<HashMap<String, HashSet<String>>>,
    pub(crate) incoming_relations: RwLock<HashMap<String, HashSet<String>>>,
}

impl Clone for KnowledgeGraph {
    fn clone(&self) -> Self {
        let entities = read_lock(&self.entities).clone();
        let entities_by_name = read_lock(&self.entities_by_name).clone();
        let entities_by_type = read_lock(&self.entities_by_type).clone();
        let relations = read_lock(&self.relations).clone();
        let outgoing_relations = read_lock(&self.outgoing_relations).clone();
        let incoming_relations = read_lock(&self.incoming_relations).clone();

        Self {
            entities: RwLock::new(entities),
            entities_by_name: RwLock::new(entities_by_name),
            entities_by_type: RwLock::new(entities_by_type),
            relations: RwLock::new(relations),
            outgoing_relations: RwLock::new(outgoing_relations),
            incoming_relations: RwLock::new(incoming_relations),
        }
    }
}

impl KnowledgeGraph {
    /// Create a new empty knowledge graph.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}
