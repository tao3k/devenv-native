use super::distance_to_score;

#[test]
fn distance_to_score_normalizes_lance_distance() {
    assert!((distance_to_score(0.0) - 1.0).abs() < f64::EPSILON);
    assert!((distance_to_score(1.0) - 0.5).abs() < f64::EPSILON);
    assert!((distance_to_score(-0.5) - 1.0).abs() < f64::EPSILON);
}
