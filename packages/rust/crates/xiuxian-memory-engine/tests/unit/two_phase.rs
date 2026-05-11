//! `TwoPhaseSearch` tests.

use std::sync::Arc;
use xiuxian_memory_engine::{
    Episode, EpisodeDraft, IntentEncoder, QTable, TwoPhaseSearch, TwoPhaseSearchRequest,
};

fn create_test_episodes() -> Vec<Episode> {
    let encoder = IntentEncoder::new(128);
    vec![
        Episode::new(EpisodeDraft {
            id: ("ep-0".to_string()).into(),
            intent: "debug network timeout".to_string(),
            intent_embedding: encoder.encode("debug network timeout"),
            experience: "Checked DNS settings".to_string(),
            outcome: "success".to_string(),
            scope: None,
        }),
        Episode::new(EpisodeDraft {
            id: ("ep-1".to_string()).into(),
            intent: "fix memory leak".to_string(),
            intent_embedding: encoder.encode("fix memory leak"),
            experience: "Found unbounded cache".to_string(),
            outcome: "success".to_string(),
            scope: None,
        }),
        Episode::new(EpisodeDraft {
            id: ("ep-2".to_string()).into(),
            intent: "handle async error".to_string(),
            intent_embedding: encoder.encode("handle async error"),
            experience: "Added error boundary".to_string(),
            outcome: "success".to_string(),
            scope: None,
        }),
        Episode::new(EpisodeDraft {
            id: ("ep-3".to_string()).into(),
            intent: "optimize database query".to_string(),
            intent_embedding: encoder.encode("optimize database query"),
            experience: "Added index".to_string(),
            outcome: "failure".to_string(),
            scope: None,
        }),
        Episode::new(EpisodeDraft {
            id: ("ep-4".to_string()).into(),
            intent: "debug network connection".to_string(),
            intent_embedding: encoder.encode("debug network connection"),
            experience: "Checked firewall".to_string(),
            outcome: "success".to_string(),
            scope: None,
        }),
    ]
}

#[test]
fn test_two_phase_search() {
    let episodes = create_test_episodes();
    let q_table = Arc::new(QTable::new());
    let encoder = Arc::new(IntentEncoder::new(128));
    let search = TwoPhaseSearch::with_defaults(q_table.clone(), encoder);

    q_table.update("ep-0", 1.0);
    q_table.update("ep-1", 0.5);
    q_table.update("ep-2", 0.2);

    let results = search.search(TwoPhaseSearchRequest {
        episodes: &episodes,
        intent: "debug network",
        k1: None,
        k2: None,
        lambda: Some(0.3),
    });

    assert!(!results.is_empty());
}

#[test]
fn test_semantic_only() {
    let episodes = create_test_episodes();
    let q_table = Arc::new(QTable::new());
    let encoder = Arc::new(IntentEncoder::new(128));
    let search = TwoPhaseSearch::with_defaults(q_table.clone(), encoder);

    let results = search.search(TwoPhaseSearchRequest {
        episodes: &episodes,
        intent: "debug network",
        k1: None,
        k2: None,
        lambda: Some(0.0),
    });

    assert!(!results.is_empty());
}

#[test]
fn test_q_only() {
    let episodes = create_test_episodes();
    let q_table = Arc::new(QTable::new());
    let encoder = Arc::new(IntentEncoder::new(128));
    let search = TwoPhaseSearch::with_defaults(q_table.clone(), encoder);

    q_table.update("ep-2", 1.0);

    let results = search.search(TwoPhaseSearchRequest {
        episodes: &episodes,
        intent: "random query",
        k1: None,
        k2: None,
        lambda: Some(1.0),
    });

    assert!(!results.is_empty());
}

#[test]
fn test_calculate_score() {
    let score = xiuxian_memory_engine::calculate_score(0.8, 0.5, 0.5);
    assert!((score - 0.65).abs() < 0.001);
}
