//! Integration-style unit tests for `xiuxian-memory-engine`.

use crate::common;

pub(super) use xiuxian_memory_engine::{
    Episode, EpisodeStore, IntentEncoder, QTable, StoreConfig, TwoPhaseConfig, TwoPhaseSearch,
};

mod incremental;
mod learning;
mod multi_hop;
mod persistence;
mod workflow;

type TestResult = std::result::Result<(), Box<dyn std::error::Error>>;

pub(super) fn test_store(name: &str) -> EpisodeStore {
    EpisodeStore::new(StoreConfig {
        path: common::test_store_path(name),
        embedding_dim: 128,
        table_name: name.to_string(),
    })
}

pub(super) fn create_test_episodes(store: &EpisodeStore) -> Vec<Episode> {
    vec![
        Episode::new(
            "ep-001".to_string(),
            "debug network timeout error".to_string(),
            store.encoder().encode("debug network timeout error"),
            "Checked DNS configuration and firewall rules".to_string(),
            "success".to_string(),
        ),
        Episode::new(
            "ep-002".to_string(),
            "fix memory leak in cache".to_string(),
            store.encoder().encode("fix memory leak in cache"),
            "Found unbounded HashMap, replaced with LRU cache".to_string(),
            "success".to_string(),
        ),
        Episode::new(
            "ep-003".to_string(),
            "handle async error properly".to_string(),
            store.encoder().encode("handle async error properly"),
            "Added trycatch and error boundary".to_string(),
            "success".to_string(),
        ),
        Episode::new(
            "ep-004".to_string(),
            "optimize slow database query".to_string(),
            store.encoder().encode("optimize slow database query"),
            "Added index but query still slow".to_string(),
            "failure".to_string(),
        ),
        Episode::new(
            "ep-005".to_string(),
            "debug connection refused".to_string(),
            store.encoder().encode("debug connection refused"),
            "Service was down, restarted it".to_string(),
            "success".to_string(),
        ),
    ]
}
