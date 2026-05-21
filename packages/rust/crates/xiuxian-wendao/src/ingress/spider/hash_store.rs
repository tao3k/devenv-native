//! `ingress::spider::hash_store` owns Wendao ingress spider hash store behavior.

use std::collections::HashSet;
use std::sync::{PoisonError, RwLock};

/// Deduplication store for crawled content hashes.
pub trait ContentHashStore: Send + Sync {
    /// Return true when hash is already present.
    fn contains_hash(&self, content_hash: &str) -> bool;
    /// Record a hash after successful assimilation.
    fn insert_hash(&self, content_hash: &str);
}

/// Thread-safe in-memory deduplication store.
#[derive(Debug, Default)]
pub struct InMemoryContentHashStore {
    hashes: RwLock<HashSet<String>>,
}

impl InMemoryContentHashStore {
    /// Construct an empty store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl ContentHashStore for InMemoryContentHashStore {
    fn contains_hash(&self, content_hash: &str) -> bool {
        self.hashes
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .contains(content_hash)
    }

    fn insert_hash(&self, content_hash: &str) {
        self.hashes
            .write()
            .unwrap_or_else(PoisonError::into_inner)
            .insert(content_hash.to_string());
    }
}
