use std::sync::Arc;

use super::{
    Episode, EpisodeDraft, EpisodeStore, IntentEncoder, QTable, TestResult, TwoPhaseConfig,
    TwoPhaseSearch, TwoPhaseSearchRequest, create_test_episodes, test_store,
};

#[test]
fn test_full_memory_workflow() -> TestResult {
    let store = test_store("episodes");
    let episodes = create_test_episodes(&store);
    for ep in &episodes {
        store.store(ep.clone())?;
    }

    assert_eq!(store.len(), 5);
    assert!((store.q_table.get_q("ep-001") - 0.5).abs() < f32::EPSILON);
    assert!((store.q_table.get_q("ep-002") - 0.5).abs() < f32::EPSILON);

    store.update_q("ep-001", 1.0);
    store.update_q("ep-002", 1.0);
    store.update_q("ep-003", 0.8);
    store.update_q("ep-004", 0.2);

    assert!(store.q_table.get_q("ep-001") > 0.5);
    assert!(store.q_table.get_q("ep-004") < 0.5);
    Ok(())
}

#[test]
fn test_two_phase_search_workflow() -> TestResult {
    let store = EpisodeStore::new(super::StoreConfig::default());
    let episodes = create_test_episodes(&store);
    for ep in &episodes {
        store.store(ep.clone())?;
    }

    store.update_q("ep-001", 1.0);
    store.update_q("ep-002", 0.5);
    store.update_q("ep-004", 0.1);

    let results = store.recall("debug network", 3);
    assert!(!results.is_empty());
    assert!(results.len() <= 3);

    let results = store.two_phase_recall("debug network", 5, 3, 0.3);
    assert!(!results.is_empty());
    assert!(results.len() <= 3);

    let results = store.two_phase_recall("debug network", 5, 3, 0.8);
    assert!(!results.is_empty());
    Ok(())
}

#[test]
fn test_two_phase_search_with_config() {
    let q_table = Arc::new(QTable::new());
    let encoder = Arc::new(IntentEncoder::new(128));
    let config = TwoPhaseConfig {
        k1: 10,
        k2: 3,
        lambda: 0.4,
    };
    let search = TwoPhaseSearch::new(q_table.clone(), encoder.clone(), config);

    let episodes = vec![
        Episode::new(EpisodeDraft {
            id: ("ep-a".to_string()).into(),
            intent: "python async programming".to_string(),
            intent_embedding: encoder.encode("python async programming"),
            experience: "Used asyncio".to_string(),
            outcome: "success".to_string(),
            scope: None,
        }),
        Episode::new(EpisodeDraft {
            id: ("ep-b".to_string()).into(),
            intent: "rust async programming".to_string(),
            intent_embedding: encoder.encode("rust async programming"),
            experience: "Used tokio".to_string(),
            outcome: "success".to_string(),
            scope: None,
        }),
        Episode::new(EpisodeDraft {
            id: ("ep-c".to_string()).into(),
            intent: "javascript callback hell".to_string(),
            intent_embedding: encoder.encode("javascript callback hell"),
            experience: "Refactored to promises".to_string(),
            outcome: "failure".to_string(),
            scope: None,
        }),
    ];

    let results = search.search(TwoPhaseSearchRequest {
        episodes: &episodes,
        intent: "async code",
        k1: None,
        k2: None,
        lambda: None,
    });
    assert!(!results.is_empty());

    let results = search.quick_search(&episodes, "async code");
    assert!(!results.is_empty());
}
