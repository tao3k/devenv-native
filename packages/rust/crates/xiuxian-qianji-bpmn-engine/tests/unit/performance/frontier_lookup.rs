use crate::test_support::MustExt as _;
use serde_json::json;
use std::collections::HashMap;
use std::hint::black_box;
use std::sync::Arc;
use std::time::Instant;
use xiuxian_qianji_bpmn_engine::{
    BpmnEdgeSpec, BpmnFrontierExecutionProposal, BpmnInstanceInit, BpmnNodeKind, BpmnNodeSpec,
    BpmnPackage, BpmnProcessSpec, ProcessKey, TokenRecord, create_instance,
};

#[test]
#[ignore = "performance probe"]
fn performance_probe_frontier_token_lookup_compares_linear_scan_vs_batch_lookup() {
    let token_count = 10_000_u64;
    let iterations = 64_u32;
    let lookup_count = 512_u64;
    let package = Arc::new(BpmnPackage::new(
        "pkg_perf_frontier_lookup",
        vec![start_end_process("frontier_lookup_probe")],
    ));
    let mut state = create_instance(
        Arc::clone(&package),
        "frontier_lookup_probe",
        BpmnInstanceInit::new("wf_frontier_lookup_probe", json!({}), 1),
    )
    .must("probe instance should exist");
    state.active_tokens = (0..token_count)
        .map(|index| TokenRecord {
            token_id: index + 1,
            node_index: 1,
            incoming_edge_index: Some((index % 8) as u32),
            inclusive_join_hint: None,
        })
        .collect();
    let lookup_ids: Vec<u64> = (0..lookup_count)
        .map(|offset| token_count - offset)
        .collect();
    let lookup_proposals = lookup_ids
        .iter()
        .map(|token_id| {
            let token_index = usize::try_from(*token_id - 1)
                .must("probe token id should fit in usize token index");
            let token = &state.active_tokens[token_index];
            BpmnFrontierExecutionProposal {
                token_id: *token_id,
                token_index,
                node_index: token.node_index,
                incoming_edge_index: token.incoming_edge_index,
            }
        })
        .collect::<Vec<_>>();

    let linear_start = Instant::now();
    let mut linear_sum = 0_usize;
    for _ in 0..iterations {
        for token_id in &lookup_ids {
            linear_sum += linear_token_index_for_id(&state.active_tokens, *token_id)
                .must("linear lookup should resolve every token");
        }
    }
    let linear_elapsed = linear_start.elapsed();

    let batch_lookup_start = Instant::now();
    let mut batch_lookup_sum = 0_usize;
    for _ in 0..iterations {
        let token_lookup = build_token_lookup(&state.active_tokens);
        for token_id in &lookup_ids {
            batch_lookup_sum += token_lookup
                .get(token_id)
                .copied()
                .must("batch lookup should resolve every token");
        }
    }
    let batch_lookup_elapsed = batch_lookup_start.elapsed();

    let proposal_index_start = Instant::now();
    let mut proposal_index_sum = 0_usize;
    for _ in 0..iterations {
        for proposal in &lookup_proposals {
            proposal_index_sum += direct_proposal_token_index(&state.active_tokens, proposal)
                .must("proposal index lookup should resolve every token");
        }
    }
    let proposal_index_elapsed = proposal_index_start.elapsed();

    assert_eq!(linear_sum, batch_lookup_sum);
    assert_eq!(linear_sum, proposal_index_sum);
    black_box((linear_sum, batch_lookup_sum, proposal_index_sum));
    eprintln!(
        "performance_probe frontier_token_lookup tokens={} lookups_per_batch={} iterations={} linear_ms={:.3} batch_lookup_ms={:.3} proposal_index_ms={:.3}",
        token_count,
        lookup_ids.len(),
        iterations,
        linear_elapsed.as_secs_f64() * 1000.0,
        batch_lookup_elapsed.as_secs_f64() * 1000.0,
        proposal_index_elapsed.as_secs_f64() * 1000.0
    );
}

#[test]
#[ignore = "performance probe"]
fn performance_probe_frontier_removal_lookup_compares_linear_scan_vs_shifted_cursor() {
    let stable_prefix_count = 4_000_u64;
    let removable_token_count = 4_000_u64;
    let iterations = 8_u32;
    let token_count = stable_prefix_count + removable_token_count;
    let active_tokens = build_frontier_snapshot_probe_tokens(token_count, 20_000);
    let proposals = active_tokens
        .iter()
        .enumerate()
        .skip(usize::try_from(stable_prefix_count).must("stable prefix count should fit in usize"))
        .map(|(token_index, token)| BpmnFrontierExecutionProposal {
            token_id: (token.token_id),
            token_index,
            node_index: token.node_index,
            incoming_edge_index: token.incoming_edge_index,
        })
        .collect::<Vec<_>>();

    let linear_start = Instant::now();
    let mut linear_sum = 0_usize;
    for _ in 0..iterations {
        let mut active_tokens = active_tokens.clone();
        for proposal in &proposals {
            let token_index = linear_token_index_for_id(&active_tokens, proposal.token_id)
                .must("linear lookup should resolve every removable token");
            linear_sum += token_index;
            active_tokens.remove(token_index);
        }
    }
    let linear_elapsed = linear_start.elapsed();

    let shifted_start = Instant::now();
    let mut shifted_sum = 0_usize;
    for _ in 0..iterations {
        let mut active_tokens = active_tokens.clone();
        for (frontier_index_shift, proposal) in proposals.iter().enumerate() {
            let token_index = proposal
                .token_index
                .checked_sub(frontier_index_shift)
                .must("shifted proposal index should stay inside the active frontier");
            assert_eq!(active_tokens[token_index].token_id, proposal.token_id);
            shifted_sum += token_index;
            active_tokens.remove(token_index);
        }
    }
    let shifted_elapsed = shifted_start.elapsed();

    assert_eq!(linear_sum, shifted_sum);
    black_box((linear_sum, shifted_sum));
    eprintln!(
        "performance_probe frontier_removal_lookup stable_prefix={} removable_tokens={} iterations={} linear_ms={:.3} shifted_cursor_ms={:.3}",
        stable_prefix_count,
        removable_token_count,
        iterations,
        linear_elapsed.as_secs_f64() * 1000.0,
        shifted_elapsed.as_secs_f64() * 1000.0
    );
}

#[test]
#[ignore = "performance probe"]
fn performance_probe_token_id_allocation_compares_repeated_scan_vs_allocator() {
    let initial_token_count = 8_000_u64;
    let pending_token_count = 512_u64;
    let pushed_token_count = 2_048_u64;
    let iterations = 16_u32;
    let active_tokens = build_frontier_snapshot_probe_tokens(initial_token_count, 20_000);
    let pending_token_ids = (0..pending_token_count)
        .map(|offset| initial_token_count + offset + 1)
        .collect::<Vec<_>>();

    let repeated_scan_start = Instant::now();
    let mut repeated_scan_sum = 0_u64;
    for _ in 0..iterations {
        repeated_scan_sum += repeated_scan_token_id_allocation_sum(
            &active_tokens,
            &pending_token_ids,
            pushed_token_count,
        );
    }
    let repeated_scan_elapsed = repeated_scan_start.elapsed();

    let allocator_start = Instant::now();
    let mut allocator_sum = 0_u64;
    for _ in 0..iterations {
        allocator_sum += allocator_token_id_allocation_sum(
            &active_tokens,
            &pending_token_ids,
            pushed_token_count,
        );
    }
    let allocator_elapsed = allocator_start.elapsed();

    assert_eq!(repeated_scan_sum, allocator_sum);
    black_box((repeated_scan_sum, allocator_sum));
    eprintln!(
        "performance_probe token_id_allocation initial_tokens={} pending_tokens={} pushed_tokens={} iterations={} repeated_scan_ms={:.3} allocator_ms={:.3}",
        initial_token_count,
        pending_token_count,
        pushed_token_count,
        iterations,
        repeated_scan_elapsed.as_secs_f64() * 1000.0,
        allocator_elapsed.as_secs_f64() * 1000.0
    );
}

fn start_end_process(process_id: &str) -> BpmnProcessSpec {
    BpmnProcessSpec::new(
        ProcessKey::new(
            "pkg_perf_lookup",
            process_id,
            format!("digest_{process_id}"),
        ),
        vec![
            BpmnNodeSpec::new(0, "start", BpmnNodeKind::StartEvent),
            BpmnNodeSpec::new(1, "end", BpmnNodeKind::EndEvent),
        ],
        vec![BpmnEdgeSpec::new(0, 1, None::<&str>)],
        Vec::new(),
    )
}

fn linear_token_index_for_id(active_tokens: &[TokenRecord], token_id: u64) -> Option<usize> {
    active_tokens
        .iter()
        .position(|token| token.token_id == token_id)
}

fn build_token_lookup(active_tokens: &[TokenRecord]) -> HashMap<u64, usize> {
    active_tokens
        .iter()
        .enumerate()
        .map(|(token_index, token)| (token.token_id, token_index))
        .collect()
}

fn direct_proposal_token_index(
    active_tokens: &[TokenRecord],
    proposal: &BpmnFrontierExecutionProposal,
) -> Option<usize> {
    active_tokens
        .get(proposal.token_index)
        .filter(|token| {
            token.token_id == proposal.token_id
                && token.node_index == proposal.node_index
                && token.incoming_edge_index == proposal.incoming_edge_index
        })
        .map(|_| proposal.token_index)
}

fn build_frontier_snapshot_probe_tokens(token_count: u64, node_count: u32) -> Vec<TokenRecord> {
    (0..token_count)
        .map(|offset| TokenRecord {
            token_id: offset + 1,
            node_index: 5 + u32::try_from(offset % u64::from(node_count - 5))
                .must("frontier snapshot probe token offset should fit in u32"),
            incoming_edge_index: Some(
                u32::try_from(offset % 8).must("frontier snapshot probe edge should fit in u32"),
            ),
            inclusive_join_hint: None,
        })
        .collect()
}

fn repeated_scan_token_id_allocation_sum(
    active_tokens: &[TokenRecord],
    pending_token_ids: &[u64],
    pushed_token_count: u64,
) -> u64 {
    let mut active_tokens = active_tokens.to_vec();
    let mut sum = 0_u64;
    for _ in 0..pushed_token_count {
        let token_id = next_probe_token_id(&active_tokens, pending_token_ids);
        sum = sum.wrapping_add(token_id);
        active_tokens.push(probe_token(token_id));
    }
    sum
}

fn allocator_token_id_allocation_sum(
    active_tokens: &[TokenRecord],
    pending_token_ids: &[u64],
    pushed_token_count: u64,
) -> u64 {
    let mut next_token_id = next_probe_token_id(active_tokens, pending_token_ids);
    let mut sum = 0_u64;
    for _ in 0..pushed_token_count {
        let token_id = next_token_id;
        next_token_id = next_token_id.saturating_add(1);
        sum = sum.wrapping_add(token_id);
    }
    sum
}

fn next_probe_token_id(active_tokens: &[TokenRecord], pending_token_ids: &[u64]) -> u64 {
    active_tokens
        .iter()
        .map(|token| token.token_id)
        .chain(pending_token_ids.iter().copied())
        .max()
        .unwrap_or(0)
        + 1
}

fn probe_token(token_id: u64) -> TokenRecord {
    TokenRecord {
        token_id,
        node_index: 1,
        incoming_edge_index: Some(0),
        inclusive_join_hint: None,
    }
}
