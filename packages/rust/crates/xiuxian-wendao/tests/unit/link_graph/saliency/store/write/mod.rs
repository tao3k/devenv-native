use super::coactivation::coactivation_weight_for_neighbor;
use super::time::unix_seconds_to_f64;
use super::types::{CoactivationNeighbor, CoactivationNeighborDirection};

#[test]
fn unix_seconds_to_f64_converts_seconds() {
    assert!((unix_seconds_to_f64(0) - 0.0).abs() < f64::EPSILON);
    assert!((unix_seconds_to_f64(86_400) - 86_400.0).abs() < f64::EPSILON);
}

#[test]
fn outbound_neighbors_are_weighted_more_than_inbound_neighbors() {
    let outbound = CoactivationNeighbor {
        node_id: "note-a".to_string(),
        direction: CoactivationNeighborDirection::Outbound,
        rank: 0,
    };
    let inbound = CoactivationNeighbor {
        node_id: "note-b".to_string(),
        direction: CoactivationNeighborDirection::Inbound,
        rank: 0,
    };

    assert!((coactivation_weight_for_neighbor(&outbound) - 1.0).abs() < f64::EPSILON);
    assert!((coactivation_weight_for_neighbor(&inbound) - 0.25).abs() < f64::EPSILON);
}

#[test]
fn higher_rank_neighbors_receive_lower_weight() {
    let rank_zero = CoactivationNeighbor {
        node_id: "note-a".to_string(),
        direction: CoactivationNeighborDirection::Outbound,
        rank: 0,
    };
    let rank_two = CoactivationNeighbor {
        node_id: "note-b".to_string(),
        direction: CoactivationNeighborDirection::Outbound,
        rank: 2,
    };

    assert!(
        coactivation_weight_for_neighbor(&rank_zero) > coactivation_weight_for_neighbor(&rank_two)
    );
}
