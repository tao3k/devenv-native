use crate::test_support::MustExt as _;
use qianji_bpmn_engine::TokenRecord;
use std::time::Instant;

#[test]
#[ignore = "performance probe"]
fn performance_probe_boundary_wait_token_retention_compares_linear_vs_indexed() {
    let boundary_token_count = 256_u64;
    let unrelated_token_count = 10_000_u64;
    let iterations = 128_u32;
    let blocking_node_index = 7_u32;
    let active_tokens = build_boundary_wait_tokens(
        boundary_token_count,
        unrelated_token_count,
        blocking_node_index,
    );
    let boundary_token_ids: Vec<u64> = active_tokens
        .iter()
        .filter(|token| token.node_index == blocking_node_index)
        .map(|token| token.token_id)
        .collect();

    let linear_start = Instant::now();
    let mut linear_winner_index_sum = 0_usize;
    let mut linear_survivor_sum = 0_usize;
    for _ in 0..iterations {
        let (winner_index, survivor_count) = linear_boundary_wait_token_retention(
            &active_tokens,
            &boundary_token_ids,
            blocking_node_index,
        );
        linear_winner_index_sum += winner_index;
        linear_survivor_sum += survivor_count;
    }
    let linear_elapsed = linear_start.elapsed();

    let indexed_start = Instant::now();
    let mut indexed_winner_index_sum = 0_usize;
    let mut indexed_survivor_sum = 0_usize;
    for _ in 0..iterations {
        let (winner_index, survivor_count) = indexed_boundary_wait_token_retention(
            &active_tokens,
            &boundary_token_ids,
            blocking_node_index,
        );
        indexed_winner_index_sum += winner_index;
        indexed_survivor_sum += survivor_count;
    }
    let indexed_elapsed = indexed_start.elapsed();

    assert_eq!(linear_winner_index_sum, indexed_winner_index_sum);
    assert_eq!(linear_survivor_sum, indexed_survivor_sum);
    eprintln!(
        "performance_probe boundary_wait_token_retention boundary_tokens={} unrelated_tokens={} iterations={} linear_ms={:.3} indexed_ms={:.3}",
        boundary_token_count,
        unrelated_token_count,
        iterations,
        linear_elapsed.as_secs_f64() * 1000.0,
        indexed_elapsed.as_secs_f64() * 1000.0
    );
}

fn build_boundary_wait_tokens(
    boundary_token_count: u64,
    unrelated_token_count: u64,
    blocking_node_index: u32,
) -> Vec<TokenRecord> {
    let capacity = usize::try_from(boundary_token_count + unrelated_token_count)
        .must("boundary probe token count should fit in usize");
    let mut active_tokens = Vec::with_capacity(capacity);
    active_tokens.extend((0..boundary_token_count).map(|offset| TokenRecord {
        token_id: boundary_token_count - offset,
        node_index: blocking_node_index,
        incoming_edge_index: Some(
            u32::try_from(offset % 8).must("boundary probe edge offset should fit in u32"),
        ),
        inclusive_join_hint: None,
    }));
    active_tokens.extend((0..unrelated_token_count).map(|offset| TokenRecord {
        token_id: boundary_token_count + offset + 1,
        node_index: 1_000 + u32::try_from(offset).must("probe offset should fit in u32"),
        incoming_edge_index: None,
        inclusive_join_hint: None,
    }));
    active_tokens
}

fn linear_boundary_wait_token_retention(
    active_tokens: &[TokenRecord],
    boundary_token_ids: &[u64],
    blocking_node_index: u32,
) -> (usize, usize) {
    let boundary_token_ids = boundary_token_ids.to_vec();
    let winning_token_id = boundary_token_ids
        .into_iter()
        .min()
        .must("linear boundary resolution should find the winner token");
    let mut surviving_tokens = active_tokens.to_vec();
    surviving_tokens.retain(|token| {
        token.token_id == winning_token_id || token.node_index != blocking_node_index
    });
    let winner_token_index = surviving_tokens
        .iter()
        .position(|token| token.token_id == winning_token_id)
        .must("linear boundary resolution should retain the winner token");
    (winner_token_index, surviving_tokens.len())
}

fn indexed_boundary_wait_token_retention(
    active_tokens: &[TokenRecord],
    boundary_token_ids: &[u64],
    blocking_node_index: u32,
) -> (usize, usize) {
    let winning_token_id = boundary_token_ids
        .iter()
        .copied()
        .min()
        .must("indexed boundary resolution should find the winner token");
    let mut winner_token_index = None;
    let mut surviving_tokens = Vec::with_capacity(active_tokens.len());
    for token in active_tokens.iter().cloned() {
        if token.token_id == winning_token_id || token.node_index != blocking_node_index {
            if token.token_id == winning_token_id {
                winner_token_index = Some(surviving_tokens.len());
            }
            surviving_tokens.push(token);
        }
    }
    (
        winner_token_index.must("indexed boundary resolution should retain the winner token"),
        surviving_tokens.len(),
    )
}
