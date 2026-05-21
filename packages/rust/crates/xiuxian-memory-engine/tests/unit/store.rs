//! `EpisodeStore` tests.

use crate::common;
use xiuxian_memory_engine::{Episode, EpisodeDraft, EpisodeStore, StoreConfig};

type TestResult = std::result::Result<(), Box<dyn std::error::Error>>;

#[test]
fn test_store_creation() {
    let path = common::test_store_path("test");
    let store = EpisodeStore::new(StoreConfig {
        path: path.clone(),
        embedding_dim: 128,
        table_name: "test".to_string(),
    });

    assert_eq!(store.len(), 0);
    assert!(store.is_empty());
}

#[test]
fn test_store_episode() -> TestResult {
    let store = EpisodeStore::default();

    let episode = Episode::new(EpisodeDraft {
        id: ("ep-001".to_string()).into(),
        intent: "debug network error".to_string(),
        intent_embedding: store.encoder().encode("debug network error"),
        experience: "Checked firewall".to_string(),
        outcome: "success".to_string(),
        scope: None,
    });

    let id = store.store(episode)?;
    assert_eq!(id, "ep-001");
    assert_eq!(store.len(), 1);
    Ok(())
}

#[test]
fn test_recall() -> TestResult {
    let store = EpisodeStore::default();

    for i in 0..5 {
        let episode = Episode::new(EpisodeDraft {
            id: (format!("ep-{i}")).into(),
            intent: format!("intent {i}"),
            intent_embedding: store.encoder().encode(&format!("intent {i}")),
            experience: format!("experience {i}"),
            outcome: "success".to_string(),
            scope: None,
        });
        store.store(episode)?;
    }

    let results = store.recall("intent 0", 3);
    assert!(results.len() <= 3);
    Ok(())
}

#[test]
fn test_two_phase_recall() -> TestResult {
    let store = EpisodeStore::default();

    for i in 0..5 {
        let episode = Episode::new(EpisodeDraft {
            id: (format!("ep-{i}")).into(),
            intent: format!("debug error {i}"),
            intent_embedding: store.encoder().encode(&format!("debug error {i}")),
            experience: format!("experience {i}"),
            outcome: if i < 3 { "success" } else { "failure" }.to_string(),
            scope: None,
        });
        store.store(episode)?;
    }

    store.update_q("ep-0", 1.0);
    store.update_q("ep-1", 0.8);
    store.update_q("ep-2", 0.3);

    let results = store.two_phase_recall("debug error", 5, 3, 0.5);
    assert!(results.len() <= 3);
    Ok(())
}

#[test]
fn test_q_update() -> TestResult {
    let store = EpisodeStore::default();

    let episode = Episode::new(EpisodeDraft {
        id: ("ep-001".to_string()).into(),
        intent: "test".to_string(),
        intent_embedding: store.encoder().encode("test"),
        experience: "experience".to_string(),
        outcome: "success".to_string(),
        scope: None,
    });
    store.store(episode)?;

    let q_initial = store.q_table.get_q("ep-001");
    assert!((q_initial - 0.5).abs() < f32::EPSILON);

    let q_new = store.update_q("ep-001", 1.0);
    assert!(q_new > 0.5);
    Ok(())
}

#[test]
fn test_recall_feedback_snapshot_roundtrip() {
    let store = EpisodeStore::default();
    store.set_recall_feedback_bias("session-1", 0.7);
    store.set_recall_feedback_bias("session-2", -0.4);

    let snapshot = store.snapshot();
    assert_eq!(
        snapshot
            .recall_feedback_bias_by_scope
            .get("session-1")
            .copied(),
        Some(0.7)
    );

    let mut restored = EpisodeStore::default();
    restored.restore_snapshot(snapshot);

    assert_eq!(restored.recall_feedback_bias("session-2"), Some(-0.4));
}
