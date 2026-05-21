#![allow(missing_docs)]

use anyhow::Result;
use xiuxian_memory_engine::{Episode, EpisodeDraft, EpisodeStore, StoreConfig};

fn new_store() -> Result<EpisodeStore> {
    let tmp = tempfile::tempdir()?;
    Ok(EpisodeStore::new(StoreConfig {
        path: tmp.path().join("memory").to_string_lossy().into_owned(),
        embedding_dim: 8,
        table_name: "feedback_tracking".to_string(),
    }))
}

fn episode(id: &str) -> Episode {
    Episode::new(EpisodeDraft {
        id: (id.to_string()).into(),
        intent: "intent".to_string(),
        intent_embedding: vec![0.1; 8],
        experience: "experience".to_string(),
        outcome: "completed".to_string(),
        scope: None,
    })
}

#[test]
fn record_feedback_updates_success_and_failure_counts() -> Result<()> {
    let store = new_store()?;
    store.store(episode("ep-1"))?;

    assert!(store.record_feedback("ep-1", true));
    assert!(store.record_feedback("ep-1", false));

    let ep = store
        .get("ep-1")
        .ok_or_else(|| anyhow::anyhow!("episode should exist"))?;
    assert_eq!(ep.retrieval_count, 2);
    assert_eq!(ep.success_count, 1);
    assert_eq!(ep.failure_count, 1);
    Ok(())
}

#[test]
fn record_feedback_returns_false_for_missing_episode() -> Result<()> {
    let store = new_store()?;
    assert!(!store.record_feedback("missing", true));
    Ok(())
}
