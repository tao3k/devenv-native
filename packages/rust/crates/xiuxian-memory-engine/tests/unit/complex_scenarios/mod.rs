//! Complex scenario tests for `xiuxian-memory-engine`.

use crate::common;

pub(super) use xiuxian_memory_engine::{Episode, EpisodeStore, StoreConfig};

mod adaptation;
mod lifecycle;
mod performance;
mod persistence;

type TestResult = std::result::Result<(), Box<dyn std::error::Error>>;

pub(super) fn test_store(name: &str) -> EpisodeStore {
    EpisodeStore::new(StoreConfig {
        path: common::test_store_path(name),
        embedding_dim: 128,
        table_name: name.to_string(),
    })
}

pub(super) fn test_store_with_dim(name: &str, embedding_dim: usize) -> EpisodeStore {
    EpisodeStore::new(StoreConfig {
        path: common::test_store_path(name),
        embedding_dim,
        table_name: name.to_string(),
    })
}
