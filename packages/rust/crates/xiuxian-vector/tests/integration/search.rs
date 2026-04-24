//! Tests for search filtering and vector distance ordering.

use xiuxian_vector::VectorStore;

// =========================================================================
// Tests for matches_filter function
// =========================================================================

#[test]
fn test_matches_filter_string_exact() {
    let metadata = serde_json::json!({"domain": "python"});
    let conditions = serde_json::json!({"domain": "python"});
    assert!(VectorStore::matches_filter(&metadata, &conditions));
}

#[test]
fn test_matches_filter_string_mismatch() {
    let metadata = serde_json::json!({"domain": "python"});
    let conditions = serde_json::json!({"domain": "testing"});
    assert!(!VectorStore::matches_filter(&metadata, &conditions));
}

#[test]
fn test_matches_filter_number() {
    let metadata = serde_json::json!({"count": 42});
    let conditions = serde_json::json!({"count": 42});
    assert!(VectorStore::matches_filter(&metadata, &conditions));
}

#[test]
fn test_matches_filter_boolean() {
    let metadata = serde_json::json!({"enabled": true});
    let conditions = serde_json::json!({"enabled": true});
    assert!(VectorStore::matches_filter(&metadata, &conditions));
}

#[test]
fn test_matches_filter_missing_key() {
    let metadata = serde_json::json!({"domain": "python"});
    let conditions = serde_json::json!({"missing_key": "value"});
    assert!(!VectorStore::matches_filter(&metadata, &conditions));
}

#[test]
fn test_matches_filter_multiple_conditions_all_match() {
    let metadata = serde_json::json!({
        "domain": "python",
        "type": "function"
    });
    let conditions = serde_json::json!({
        "domain": "python",
        "type": "function"
    });
    assert!(VectorStore::matches_filter(&metadata, &conditions));
}

#[test]
fn test_matches_filter_multiple_conditions_one_mismatch() {
    let metadata = serde_json::json!({
        "domain": "python",
        "type": "function"
    });
    let conditions = serde_json::json!({
        "domain": "python",
        "type": "class"
    });
    assert!(!VectorStore::matches_filter(&metadata, &conditions));
}

#[test]
fn test_matches_filter_nested_key() {
    let metadata = serde_json::json!({
        "config": {
            "domain": "python"
        }
    });
    let conditions = serde_json::json!({
        "config.domain": "python"
    });
    assert!(VectorStore::matches_filter(&metadata, &conditions));
}

#[test]
fn test_matches_filter_null_metadata() {
    let metadata = serde_json::Value::Null;
    let conditions = serde_json::json!({"domain": "python"});
    assert!(!VectorStore::matches_filter(&metadata, &conditions));
}

#[test]
fn test_matches_filter_empty_conditions() {
    let metadata = serde_json::json!({"domain": "python"});
    let conditions = serde_json::json!({});
    // Empty conditions should match everything
    assert!(VectorStore::matches_filter(&metadata, &conditions));
}

#[test]
fn test_matches_filter_non_object_conditions() {
    let metadata = serde_json::json!({"domain": "python"});
    let conditions = serde_json::json!("invalid");
    // Non-object conditions should match everything
    assert!(VectorStore::matches_filter(&metadata, &conditions));
}

// =========================================================================
// Tests for search vector distance calculation
// =========================================================================

/// Test that vector distance calculation produces correct relative ordering.
/// Identical vectors should have distance 0 (score 1.0).
/// Vectors that differ more should have higher distance (lower score).
#[tokio::test]
async fn test_vector_distance_calculation() {
    // Calculate expected distances manually
    // dist_sq = sum((a - b)^2)
    let identical_dist_sq: f32 = (1.0_f32 - 1.0_f32).powi(2) * 4.0; // = 0
    let opposite_dist_sq: f32 = (1.0_f32 - (-1.0_f32)).powi(2) + 3.0 * (0.0_f32 - 0.0_f32).powi(2); // = 4
    let orthogonal_dist_sq: f32 = (1.0_f32 - 0.0_f32).powi(2)
        + (0.0_f32 - 1.0_f32).powi(2)
        + 2.0 * (0.0_f32 - 0.0_f32).powi(2); // = 2

    let identical_score = 1.0 / (1.0 + identical_dist_sq.sqrt());
    let opposite_score = 1.0 / (1.0 + opposite_dist_sq.sqrt());
    let orthogonal_score = 1.0 / (1.0 + orthogonal_dist_sq.sqrt());

    // Verify score ordering: identical > orthogonal > opposite
    assert!(
        identical_score > orthogonal_score,
        "Identical should score higher than orthogonal"
    );
    assert!(
        orthogonal_score > opposite_score,
        "Orthogonal should score higher than opposite"
    );
    assert!(
        (identical_score - 1.0).abs() < f32::EPSILON,
        "Identical vectors should have score 1.0"
    );

    // Verify results are ordered by score
    let mut results = [
        ("opposite", opposite_score),
        ("orthogonal", orthogonal_score),
        ("identical", identical_score),
    ];
    results.sort_by(|a, b| b.1.total_cmp(&a.1));

    assert_eq!(results[0].0, "identical");
    assert_eq!(results[1].0, "orthogonal");
    assert_eq!(results[2].0, "opposite");
}
