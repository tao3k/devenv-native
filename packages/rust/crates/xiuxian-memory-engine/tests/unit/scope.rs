//! Scope isolation tests for `EpisodeStore`.

use crate::common;
use xiuxian_memory_engine::{
    Episode, EpisodeDraft, EpisodeStore, ScopedTwoPhaseEmbeddingRecallRequest, StoreConfig,
};

const SCOPE_A: &str = "telegram:-100:111";
const SCOPE_B: &str = "telegram:-100:222";
const WRONG_SCOPE: &str = "legacy-scope";

type TestResult = std::result::Result<(), Box<dyn std::error::Error>>;

fn build_store(test_name: &str) -> EpisodeStore {
    EpisodeStore::new(StoreConfig {
        path: common::test_store_path(test_name),
        embedding_dim: 4,
        table_name: test_name.to_string(),
    })
}

#[test]
fn scoped_recall_excludes_other_sessions() -> TestResult {
    let store = build_store("scope_isolation");
    let embedding_a = vec![1.0, 0.0, 0.0, 0.0];
    let embedding_b = vec![0.0, 1.0, 0.0, 0.0];

    store.store(Episode::new(EpisodeDraft {
        id: ("ep-a".to_string()).into(),
        intent: "session a intent".to_string(),
        intent_embedding: embedding_a.clone(),
        experience: "session a response".to_string(),
        outcome: "completed".to_string(),
        scope: Some((SCOPE_A).to_string()),
    }))?;
    store.store(Episode::new(EpisodeDraft {
        id: ("ep-b".to_string()).into(),
        intent: "session b intent".to_string(),
        intent_embedding: embedding_b,
        experience: "session b response".to_string(),
        outcome: "completed".to_string(),
        scope: Some((SCOPE_B).to_string()),
    }))?;

    let scoped =
        store.two_phase_recall_with_embedding_for_scope(ScopedTwoPhaseEmbeddingRecallRequest {
            scope: SCOPE_A,
            embedding: &embedding_a,
            k1: 8,
            k2: 8,
            lambda: 0.0,
        });
    assert_eq!(
        scoped.len(),
        1,
        "scoped recall must only include matching scope"
    );
    assert_eq!(scoped[0].0.id, "ep-a");
    assert_eq!(scoped[0].0.scope_key(), SCOPE_A);

    let global = store.two_phase_recall_with_embedding(&embedding_a, 8, 8, 0.0);
    assert_eq!(global.len(), 2, "global recall still sees all scopes");
    Ok(())
}

#[test]
fn store_for_scope_overrides_episode_scope() -> TestResult {
    let store = build_store("scope_override");
    let embedding = vec![1.0, 0.0, 0.0, 0.0];

    store.store_for_scope(
        SCOPE_A,
        Episode::new(EpisodeDraft {
            id: ("ep-override".to_string()).into(),
            intent: "intent".to_string(),
            intent_embedding: embedding,
            experience: "experience".to_string(),
            outcome: "completed".to_string(),
            scope: Some((WRONG_SCOPE).to_string()),
        }),
    )?;

    let stored = store
        .get("ep-override")
        .ok_or_else(|| std::io::Error::other("episode should be persisted after scoped store"))?;
    assert_eq!(stored.scope_key(), SCOPE_A);
    Ok(())
}

#[test]
fn global_episodes_are_not_returned_by_scoped_recall() -> TestResult {
    let store = build_store("scope_global_exclusion");

    store.store(Episode::new(EpisodeDraft {
        id: ("ep-global".to_string()).into(),
        intent: "same intent".to_string(),
        intent_embedding: vec![1.0, 0.0, 0.0, 0.0],
        experience: "global episode".to_string(),
        outcome: "completed".to_string(),
        scope: None,
    }))?;

    let scoped = store.recall_for_scope(SCOPE_A, "same intent", 8);
    assert!(
        scoped.is_empty(),
        "session-scoped recall must not include global or legacy episodes"
    );
    Ok(())
}

#[test]
fn legacy_agent_episode_ids_are_auto_scoped() -> TestResult {
    let store = build_store("scope_legacy_id_inference");
    let legacy_scope = "telegram:-200:7001";
    let legacy_id = format!("turn-{legacy_scope}-1700000000123");
    let embedding = vec![1.0, 0.0, 0.0, 0.0];

    // Legacy payloads may not contain scope but keep scoped ids.
    store.store(Episode::new(EpisodeDraft {
        id: (legacy_id).into(),
        intent: "legacy scoped intent".to_string(),
        intent_embedding: embedding.clone(),
        experience: "legacy response".to_string(),
        outcome: "completed".to_string(),
        scope: None,
    }))?;

    let scoped =
        store.two_phase_recall_with_embedding_for_scope(ScopedTwoPhaseEmbeddingRecallRequest {
            scope: legacy_scope,
            embedding: &embedding,
            k1: 8,
            k2: 8,
            lambda: 0.0,
        });
    assert_eq!(scoped.len(), 1, "legacy id scope should be inferred");
    assert_eq!(scoped[0].0.scope_key(), legacy_scope);
    Ok(())
}
