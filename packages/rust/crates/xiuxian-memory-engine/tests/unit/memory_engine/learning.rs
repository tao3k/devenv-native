use super::{Episode, IntentEncoder, QTable};

#[test]
fn test_q_learning_convergence() {
    let q_table = QTable::with_params(0.1, 0.95);

    for _ in 0..100 {
        q_table.update("ep-test", 1.0);
    }

    let q_value = q_table.get_q("ep-test");
    assert!(
        (q_value - 1.0).abs() < 0.01,
        "Q-value should converge to 1.0, got {q_value}"
    );

    for _ in 0..100 {
        q_table.update("ep-test-2", 0.0);
    }

    let q_value = q_table.get_q("ep-test-2");
    assert!(
        (q_value - 0.0).abs() < 0.01,
        "Q-value should converge to 0.0, got {q_value}"
    );
}

#[test]
fn test_intent_encoder_determinism() {
    let encoder = IntentEncoder::new(128);
    let emb1 = encoder.encode("test intent query");
    let emb2 = encoder.encode("test intent query");

    assert_eq!(emb1, emb2, "Same intent should produce same embedding");

    let norm: f32 = emb1.iter().map(|x| x * x).sum::<f32>().sqrt();
    assert!((norm - 1.0).abs() < 0.001, "Embedding should be normalized");
}

#[test]
fn test_episode_utility_calculation() {
    let mut episode = Episode::new(
        "ep-test".to_string(),
        "test intent".to_string(),
        vec![0.1, 0.2, 0.3],
        "test experience".to_string(),
        "success".to_string(),
    );

    let initial_util = episode.utility();
    assert!(initial_util > 0.0);

    episode.mark_success();
    episode.mark_success();
    assert_eq!(episode.retrieval_count, 2);
    assert_eq!(episode.success_count, 2);

    episode.mark_failure();
    assert_eq!(episode.retrieval_count, 3);
    assert_eq!(episode.failure_count, 1);

    let util = episode.utility();
    assert!(util > 0.0);
}

#[test]
fn test_batch_operations() {
    let q_table = QTable::new();
    let updates = vec![
        ("ep-1".to_string(), 1.0),
        ("ep-2".to_string(), 0.8),
        ("ep-3".to_string(), 0.6),
        ("ep-4".to_string(), 0.4),
        ("ep-5".to_string(), 0.2),
    ];

    let results = q_table.update_batch(&updates);
    assert_eq!(results.len(), 5);

    for (id, _) in &updates {
        assert!(q_table.get_q(id) > 0.0);
    }
}

#[test]
fn test_calculate_score_function() {
    use xiuxian_memory_engine::calculate_score;

    let score = calculate_score(0.9, 0.5, 0.0);
    assert!((score - 0.9).abs() < 0.001);

    let score = calculate_score(0.9, 0.5, 1.0);
    assert!((score - 0.5).abs() < 0.001);

    let score = calculate_score(0.9, 0.5, 0.5);
    assert!((score - 0.7).abs() < 0.001);
}
