//! Unit tests for ccs module.

use super::{CCS_THRESHOLD, audit_search_payload, evaluate_alignment};

#[test]
fn test_ccs_perfect_alignment() {
    let anchors = vec!["latency".to_string(), "memory".to_string()];
    let evidence = vec![
        "System latency is low".to_string(),
        "Memory usage is optimized".to_string(),
    ];

    let result = audit_search_payload(&evidence, &anchors);

    assert!(result.passed);
    assert!(result.ccs_score >= 0.99);
    assert!(result.missing_anchors.is_empty());
    assert!(result.compensation.is_none());
}

#[test]
fn test_ccs_partial_alignment() {
    let anchors = vec![
        "latency".to_string(),
        "memory".to_string(),
        "throughput".to_string(),
    ];
    let evidence = vec!["Memory is full.".to_string()];

    let result = audit_search_payload(&evidence, &anchors);

    // CCS = 1/3 = 0.33 < 0.70 -> FAIL
    assert!(!result.passed);
    assert!(result.ccs_score < CCS_THRESHOLD);
    assert!(result.missing_anchors.contains(&"latency".to_string()));
    assert!(result.missing_anchors.contains(&"throughput".to_string()));
    assert!(result.compensation.is_some());
}

#[test]
fn test_ccs_empty_anchors() {
    let evidence = vec!["Some content".to_string()];

    let result = audit_search_payload(&evidence, &[]);

    // Empty anchors = perfect score (nothing to miss)
    assert!(result.passed);
    assert!((result.ccs_score - 1.0).abs() < f64::EPSILON);
}

#[test]
fn test_drift_score_calculation() {
    let anchors = vec![
        "latency".to_string(),
        "memory".to_string(),
        "throughput".to_string(),
    ];
    let evidence = vec!["contains latency".to_string()]; // 1/3 match

    let verdict = evaluate_alignment(&anchors, &evidence);

    // drift = 1 - (1/3) ≈ 0.667
    // Use f32 for comparison since drift_score is f32
    let expected_drift: f32 = 1.0 - 1.0 / 3.0;
    assert!(
        (verdict.drift_score - expected_drift).abs() < 0.01,
        "drift_score = {}, expected = {}",
        verdict.drift_score,
        expected_drift
    );
    assert!(!verdict.is_aligned);
}
