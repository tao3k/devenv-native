//! Integration-style unit tests for `xiuxian-memory-engine`.

use crate::common;

pub(super) use xiuxian_memory_engine::{
    Episode, EpisodeDraft, EpisodeStore, IntentEncoder, QTable, StoreConfig, TwoPhaseConfig,
    TwoPhaseSearch, TwoPhaseSearchRequest,
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
        Episode::new(EpisodeDraft {
            id: ("ep-001".to_string()).into(),
            intent: "debug network timeout error".to_string(),
            intent_embedding: store.encoder().encode("debug network timeout error"),
            experience: "Checked DNS configuration and firewall rules".to_string(),
            outcome: "success".to_string(),
            scope: None,
        }),
        Episode::new(EpisodeDraft {
            id: ("ep-002".to_string()).into(),
            intent: "fix memory leak in cache".to_string(),
            intent_embedding: store.encoder().encode("fix memory leak in cache"),
            experience: "Found unbounded HashMap, replaced with LRU cache".to_string(),
            outcome: "success".to_string(),
            scope: None,
        }),
        Episode::new(EpisodeDraft {
            id: ("ep-003".to_string()).into(),
            intent: "handle async error properly".to_string(),
            intent_embedding: store.encoder().encode("handle async error properly"),
            experience: "Added trycatch and error boundary".to_string(),
            outcome: "success".to_string(),
            scope: None,
        }),
        Episode::new(EpisodeDraft {
            id: ("ep-004".to_string()).into(),
            intent: "optimize slow database query".to_string(),
            intent_embedding: store.encoder().encode("optimize slow database query"),
            experience: "Added index but query still slow".to_string(),
            outcome: "failure".to_string(),
            scope: None,
        }),
        Episode::new(EpisodeDraft {
            id: ("ep-005".to_string()).into(),
            intent: "debug connection refused".to_string(),
            intent_embedding: store.encoder().encode("debug connection refused"),
            experience: "Service was down, restarted it".to_string(),
            outcome: "success".to_string(),
            scope: None,
        }),
    ]
}
