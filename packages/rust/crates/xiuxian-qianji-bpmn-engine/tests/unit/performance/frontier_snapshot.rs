use super::frontier_snapshot_data::{
    build_frontier_probe_node_statuses, build_frontier_snapshot_boundary_blocking_nodes,
    build_frontier_snapshot_probe_tokens, build_frontier_snapshot_waiting_nodes,
};
use super::frontier_snapshot_strategies::{
    adaptive_frontier_snapshot_classification_sum, dense_frontier_snapshot_classification_sum,
    direct_frontier_snapshot_classification_sum, hashset_frontier_snapshot_classification_sum,
};
use std::time::Instant;

#[test]
#[ignore = "performance probe"]
fn performance_probe_frontier_snapshot_classification_compares_hashset_dense_and_direct_status() {
    let node_count = 20_000_u32;
    let token_count = 10_000_u64;
    let wait_count = 512_u32;
    let iterations = 128_u32;
    let node_statuses = build_frontier_probe_node_statuses(node_count);
    let active_tokens = build_frontier_snapshot_probe_tokens(token_count, node_count);
    let waiting_node_indices = build_frontier_snapshot_waiting_nodes(wait_count);
    let boundary_blocking_node_indices =
        build_frontier_snapshot_boundary_blocking_nodes(wait_count, node_count);

    let hashset_start = Instant::now();
    let mut hashset_sum = 0_u64;
    for _ in 0..iterations {
        hashset_sum += hashset_frontier_snapshot_classification_sum(
            &active_tokens,
            &node_statuses,
            &waiting_node_indices,
            &boundary_blocking_node_indices,
        );
    }
    let hashset_elapsed = hashset_start.elapsed();

    let dense_start = Instant::now();
    let mut dense_sum = 0_u64;
    for _ in 0..iterations {
        dense_sum += dense_frontier_snapshot_classification_sum(
            &active_tokens,
            &node_statuses,
            &waiting_node_indices,
            &boundary_blocking_node_indices,
        );
    }
    let dense_elapsed = dense_start.elapsed();

    let direct_start = Instant::now();
    let mut direct_sum = 0_u64;
    for _ in 0..iterations {
        direct_sum += direct_frontier_snapshot_classification_sum(
            &active_tokens,
            &node_statuses,
            &waiting_node_indices,
            &boundary_blocking_node_indices,
        );
    }
    let direct_elapsed = direct_start.elapsed();

    let adaptive_start = Instant::now();
    let mut adaptive_sum = 0_u64;
    for _ in 0..iterations {
        adaptive_sum += adaptive_frontier_snapshot_classification_sum(
            &active_tokens,
            &node_statuses,
            &waiting_node_indices,
            &boundary_blocking_node_indices,
        );
    }
    let adaptive_elapsed = adaptive_start.elapsed();

    assert_eq!(hashset_sum, dense_sum);
    assert_eq!(hashset_sum, direct_sum);
    assert_eq!(hashset_sum, adaptive_sum);
    eprintln!(
        "performance_probe frontier_snapshot_classification nodes={} tokens={} waits={} iterations={} hashset_ms={:.3} dense_status_ms={:.3} sparse_direct_status_ms={:.3} adaptive_direct_status_ms={:.3}",
        node_count,
        token_count,
        wait_count,
        iterations,
        hashset_elapsed.as_secs_f64() * 1000.0,
        dense_elapsed.as_secs_f64() * 1000.0,
        direct_elapsed.as_secs_f64() * 1000.0,
        adaptive_elapsed.as_secs_f64() * 1000.0
    );
}
