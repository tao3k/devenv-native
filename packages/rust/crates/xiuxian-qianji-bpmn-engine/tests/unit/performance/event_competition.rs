use crate::test_support::MustExt as _;
use std::collections::HashSet;
use std::hint::black_box;
use std::time::Instant;
use xiuxian_qianji_bpmn_engine::TokenRecord;

#[test]
#[ignore = "performance probe"]
fn performance_probe_event_competition_wait_resolution_compares_linear_vs_indexed() {
    let wait_count = 64_u32;
    let unrelated_token_count = 10_000_u32;
    let iterations = 128_u32;
    let winning_wait_node_index = wait_count + 1;
    let active_tokens = build_event_competition_tokens(wait_count, unrelated_token_count);
    let wait_node_indices: Vec<u32> = (2..2 + wait_count).collect();

    let linear_start = Instant::now();
    let mut linear_winner_index_sum = 0_usize;
    let mut linear_survivor_sum = 0_usize;
    for _ in 0..iterations {
        let (winner_index, survivor_count, retained_wait_count) =
            linear_event_competition_resolution(
                &active_tokens,
                &wait_node_indices,
                winning_wait_node_index,
            );
        linear_winner_index_sum += winner_index;
        linear_survivor_sum += survivor_count + retained_wait_count;
    }
    let linear_elapsed = linear_start.elapsed();

    let indexed_start = Instant::now();
    let mut indexed_winner_index_sum = 0_usize;
    let mut indexed_survivor_sum = 0_usize;
    for _ in 0..iterations {
        let (winner_index, survivor_count, retained_wait_count) =
            indexed_event_competition_resolution(
                &active_tokens,
                &wait_node_indices,
                winning_wait_node_index,
            );
        indexed_winner_index_sum += winner_index;
        indexed_survivor_sum += survivor_count + retained_wait_count;
    }
    let indexed_elapsed = indexed_start.elapsed();

    let fused_indexed_start = Instant::now();
    let mut fused_indexed_winner_index_sum = 0_usize;
    let mut fused_indexed_survivor_sum = 0_usize;
    for _ in 0..iterations {
        let (winner_index, survivor_count, retained_wait_count) =
            fused_indexed_event_competition_resolution(
                &active_tokens,
                &wait_node_indices,
                winning_wait_node_index,
            );
        fused_indexed_winner_index_sum += winner_index;
        fused_indexed_survivor_sum += survivor_count + retained_wait_count;
    }
    let fused_indexed_elapsed = fused_indexed_start.elapsed();

    assert_eq!(linear_winner_index_sum, indexed_winner_index_sum);
    assert_eq!(linear_winner_index_sum, fused_indexed_winner_index_sum);
    assert_eq!(linear_survivor_sum, indexed_survivor_sum);
    assert_eq!(linear_survivor_sum, fused_indexed_survivor_sum);
    black_box((
        linear_winner_index_sum,
        indexed_winner_index_sum,
        fused_indexed_winner_index_sum,
        linear_survivor_sum,
        indexed_survivor_sum,
        fused_indexed_survivor_sum,
    ));
    eprintln!(
        "performance_probe event_competition_wait_resolution waits={} unrelated_tokens={} iterations={} linear_ms={:.3} indexed_ms={:.3} fused_indexed_ms={:.3}",
        wait_count,
        unrelated_token_count,
        iterations,
        linear_elapsed.as_secs_f64() * 1000.0,
        indexed_elapsed.as_secs_f64() * 1000.0,
        fused_indexed_elapsed.as_secs_f64() * 1000.0
    );
}

fn build_event_competition_tokens(wait_count: u32, unrelated_token_count: u32) -> Vec<TokenRecord> {
    let mut active_tokens = Vec::with_capacity((wait_count + unrelated_token_count) as usize);
    active_tokens.extend((0..wait_count).map(|offset| TokenRecord {
        token_id: (u64::from(offset) + 1),
        node_index: 2 + offset,
        incoming_edge_index: Some(offset),
        inclusive_join_hint: None,
    }));
    active_tokens.extend((0..unrelated_token_count).map(|offset| TokenRecord {
        token_id: (u64::from(wait_count + offset) + 1),
        node_index: 1_000 + offset,
        incoming_edge_index: None,
        inclusive_join_hint: None,
    }));
    active_tokens
}

fn linear_event_competition_resolution(
    active_tokens: &[TokenRecord],
    wait_node_indices: &[u32],
    winning_wait_node_index: u32,
) -> (usize, usize, usize) {
    let winner_token_id = active_tokens
        .iter()
        .find(|token| token.node_index == winning_wait_node_index)
        .must("linear resolution should find the winner token")
        .token_id;
    let mut surviving_tokens = active_tokens.to_vec();
    surviving_tokens.retain(|token| {
        token.token_id == winner_token_id || !wait_node_indices.contains(&token.node_index)
    });
    let winner_token_index = surviving_tokens
        .iter()
        .position(|token| token.token_id == winner_token_id)
        .must("linear resolution should retain the winner token");
    let mut retained_wait_node_indices = wait_node_indices.to_vec();
    retained_wait_node_indices
        .retain(|wait_node_index| !wait_node_indices.contains(wait_node_index));
    (
        winner_token_index,
        surviving_tokens.len(),
        retained_wait_node_indices.len(),
    )
}

fn indexed_event_competition_resolution(
    active_tokens: &[TokenRecord],
    wait_node_indices: &[u32],
    winning_wait_node_index: u32,
) -> (usize, usize, usize) {
    let wait_node_index_set: HashSet<u32> = wait_node_indices.iter().copied().collect();
    let winner_token_id = active_tokens
        .iter()
        .find(|token| token.node_index == winning_wait_node_index)
        .must("indexed resolution should find the winner token")
        .token_id;
    let mut winner_token_index = None;
    let mut surviving_tokens = Vec::with_capacity(active_tokens.len());
    for token in active_tokens.iter().cloned() {
        if token.token_id == winner_token_id || !wait_node_index_set.contains(&token.node_index) {
            if token.token_id == winner_token_id {
                winner_token_index = Some(surviving_tokens.len());
            }
            surviving_tokens.push(token);
        }
    }
    let mut retained_wait_node_indices = wait_node_indices.to_vec();
    retained_wait_node_indices
        .retain(|wait_node_index| !wait_node_index_set.contains(wait_node_index));
    (
        winner_token_index.must("indexed resolution should retain the winner token"),
        surviving_tokens.len(),
        retained_wait_node_indices.len(),
    )
}

fn fused_indexed_event_competition_resolution(
    active_tokens: &[TokenRecord],
    wait_node_indices: &[u32],
    winning_wait_node_index: u32,
) -> (usize, usize, usize) {
    let wait_node_index_set: HashSet<u32> = wait_node_indices.iter().copied().collect();
    let mut winner_token_index = None;
    let mut surviving_tokens = Vec::with_capacity(active_tokens.len());
    for token in active_tokens.iter().cloned() {
        if winner_token_index.is_none() && token.node_index == winning_wait_node_index {
            winner_token_index = Some(surviving_tokens.len());
            surviving_tokens.push(token);
        } else if !wait_node_index_set.contains(&token.node_index) {
            surviving_tokens.push(token);
        }
    }
    let mut retained_wait_node_indices = wait_node_indices.to_vec();
    retained_wait_node_indices
        .retain(|wait_node_index| !wait_node_index_set.contains(wait_node_index));
    (
        winner_token_index.must("fused indexed resolution should retain the winner token"),
        surviving_tokens.len(),
        retained_wait_node_indices.len(),
    )
}
