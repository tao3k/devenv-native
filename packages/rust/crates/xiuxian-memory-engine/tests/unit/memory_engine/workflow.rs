use std::sync::Arc;

use super::{
    Episode, EpisodeStore, IntentEncoder, QTable, TestResult, TwoPhaseConfig, TwoPhaseSearch,
    create_test_episodes, test_store,
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
        Episode::new(
            "ep-a".to_string(),
            "python async programming".to_string(),
            encoder.encode("python async programming"),
            "Used asyncio".to_string(),
            "success".to_string(),
        ),
        Episode::new(
            "ep-b".to_string(),
            "rust async programming".to_string(),
            encoder.encode("rust async programming"),
            "Used tokio".to_string(),
            "success".to_string(),
        ),
        Episode::new(
            "ep-c".to_string(),
            "javascript callback hell".to_string(),
            encoder.encode("javascript callback hell"),
            "Refactored to promises".to_string(),
            "failure".to_string(),
        ),
    ];

    let results = search.search(&episodes, "async code", None, None, None);
    assert!(!results.is_empty());

    let results = search.quick_search(&episodes, "async code");
    assert!(!results.is_empty());
}
