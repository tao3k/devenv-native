use super::super::{
    RerankScoreWeights, must_err, must_ok, score_rerank_request_batch,
    score_rerank_request_batch_with_weights,
};
use super::support::build_rerank_request_batch;

#[test]
fn rerank_request_batch_scoring_blends_vector_and_semantic_similarity() {
    let batch = build_rerank_request_batch(
        vec!["doc-0", "doc-1"],
        vec![0.5_f32, 0.8_f32],
        vec![
            vec![1.0_f32, 0.0_f32, 0.0_f32],
            vec![0.0_f32, 1.0_f32, 0.0_f32],
        ],
        vec![
            vec![1.0_f32, 0.0_f32, 0.0_f32],
            vec![1.0_f32, 0.0_f32, 0.0_f32],
        ],
    );

    let scored = must_ok(
        score_rerank_request_batch(&batch, 3),
        "rerank scoring should succeed",
    );

    assert_eq!(scored.len(), 2);
    assert_eq!(scored[0].doc_id, "doc-0");
    assert!((scored[0].vector_score - 0.5).abs() < 1e-6);
    assert!((scored[0].semantic_score - 1.0).abs() < 1e-6);
    assert!((scored[0].final_score - 0.8).abs() < 1e-6);
    assert_eq!(scored[1].doc_id, "doc-1");
    assert!((scored[1].vector_score - 0.8).abs() < 1e-6);
    assert!((scored[1].semantic_score - 0.5).abs() < 1e-6);
    assert!((scored[1].final_score - 0.62).abs() < 1e-6);
}

#[test]
fn rerank_score_weights_normalize_runtime_policy() {
    let weights = must_ok(RerankScoreWeights::new(2.0, 3.0), "weights should validate");
    let normalized = weights.normalized();

    assert!((normalized.vector_weight - 0.4).abs() < 1e-6);
    assert!((normalized.semantic_weight - 0.6).abs() < 1e-6);
}

#[test]
fn rerank_score_weights_reject_zero_sum_policy() {
    let error = must_err(
        RerankScoreWeights::new(0.0, 0.0),
        "zero-sum weights should fail",
    );
    assert_eq!(error, "rerank score weights must sum to greater than zero");
}

#[test]
fn score_rerank_request_batch_with_weights_respects_runtime_policy() {
    let batch = build_rerank_request_batch(
        vec!["doc-0", "doc-1"],
        vec![0.5_f32, 0.8_f32],
        vec![
            vec![1.0_f32, 0.0_f32, 0.0_f32],
            vec![0.0_f32, 1.0_f32, 0.0_f32],
        ],
        vec![
            vec![1.0_f32, 0.0_f32, 0.0_f32],
            vec![1.0_f32, 0.0_f32, 0.0_f32],
        ],
    );

    let scored = must_ok(
        score_rerank_request_batch_with_weights(
            &batch,
            3,
            must_ok(RerankScoreWeights::new(0.9, 0.1), "weights should validate"),
        ),
        "rerank scoring should succeed",
    );

    assert!((scored[0].final_score - 0.55).abs() < 1e-6);
    assert!((scored[1].final_score - 0.77).abs() < 1e-6);
    assert!(scored[1].final_score > scored[0].final_score);
}
