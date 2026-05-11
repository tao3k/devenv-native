//! `Episode` tests.

use xiuxian_memory_engine::{Episode, EpisodeDraft};

#[test]
fn test_episode_creation() {
    let episode = Episode::new(EpisodeDraft {
        id: ("ep-001".to_string()).into(),
        intent: "debug network error".to_string(),
        intent_embedding: vec![0.1, 0.2, 0.3],
        experience: "Checked firewall rules".to_string(),
        outcome: "success".to_string(),
        scope: None,
    });

    assert_eq!(episode.id, "ep-001");
    assert!((episode.q_value - 0.5).abs() < f32::EPSILON);
    assert_eq!(episode.retrieval_count, 0);
    assert_eq!(episode.success_count, 0);
    assert_eq!(episode.failure_count, 0);
    assert_eq!(episode.created_at, episode.updated_at);
}

#[test]
fn test_utility_calculation() {
    let mut episode = Episode::new(EpisodeDraft {
        id: ("ep-001".to_string()).into(),
        intent: "test intent".to_string(),
        intent_embedding: vec![0.1, 0.2],
        experience: "test experience".to_string(),
        outcome: "success".to_string(),
        scope: None,
    });

    let initial_util = episode.utility();
    assert!(initial_util > 0.0);

    episode.mark_success();
    assert_eq!(episode.retrieval_count, 1);
    assert_eq!(episode.success_count, 1);

    episode.mark_failure();
    assert_eq!(episode.retrieval_count, 2);
    assert_eq!(episode.failure_count, 1);
    assert_eq!(episode.total_uses(), 2);
}
