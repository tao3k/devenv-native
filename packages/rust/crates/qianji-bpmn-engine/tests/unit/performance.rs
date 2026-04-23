use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{
    BpmnEdgeSpec, BpmnFrontierEntryStatus, BpmnFrontierExecutionProposal,
    BpmnFrontierExecutionStep, BpmnFrontierPlanAction, BpmnIndexRange, BpmnInstanceInit,
    BpmnInstanceState, BpmnNodeKind, BpmnNodeSpec, BpmnPackage, BpmnProcessSpec, NodeRuntimeStatus,
    ProcessKey, TokenRecord, create_instance, merge_frontier_execution_steps, plan_frontier_step,
};
use serde_json::json;
use std::collections::{HashMap, HashSet};
use std::hint::black_box;
use std::sync::Arc;
use std::time::Instant;

const PROBE_DENSE_WAIT_ROLE_THRESHOLD: usize = 32;
const PROBE_DENSE_WAIT_ROLE_TOKEN_THRESHOLD: usize = 128;
const PROBE_DENSE_WAIT_ROLE_NODE_TO_TOKEN_RATIO: usize = 8;

#[test]
#[ignore = "performance probe"]
fn performance_probe_process_lookup_cache_compares_linear_scan_vs_index_access() {
    let process_count = 20_000_u32;
    let iterations = 200_000_u32;
    let target_process_id = format!("proc_{}", process_count - 1);
    let package = Arc::new(BpmnPackage::new(
        "pkg_perf_lookup",
        (0..process_count)
            .map(|index| start_end_process(&format!("proc_{index}")))
            .collect(),
    ));
    let state = create_instance(
        Arc::clone(&package),
        &target_process_id,
        BpmnInstanceInit::new("wf_perf_lookup", json!({}), 1),
    )
    .must("target process should exist");

    let linear_start = Instant::now();
    let mut linear_nodes = 0_usize;
    for _ in 0..iterations {
        let process = package
            .find_process(&target_process_id)
            .must("linear lookup should find the process");
        linear_nodes += process.nodes.len();
    }
    let linear_elapsed = linear_start.elapsed();

    let indexed_start = Instant::now();
    let mut indexed_nodes = 0_usize;
    for _ in 0..iterations {
        let process = &package.processes[state.process_index as usize];
        indexed_nodes += process.nodes.len();
    }
    let indexed_elapsed = indexed_start.elapsed();

    assert_eq!(linear_nodes, indexed_nodes);
    eprintln!(
        "performance_probe process_lookup processes={} iterations={} linear_ms={:.3} indexed_ms={:.3}",
        process_count,
        iterations,
        linear_elapsed.as_secs_f64() * 1000.0,
        indexed_elapsed.as_secs_f64() * 1000.0
    );
}

#[test]
#[ignore = "performance probe"]
fn performance_probe_wait_process_lookup_compares_linear_vs_indexed() {
    let process_count = 20_000_u32;
    let iterations = 200_000_u32;
    let target_process_index = process_count - 1;
    let target_process_id = format!("proc_{target_process_index}");
    let package = BpmnPackage::new(
        "pkg_perf_wait_lookup",
        (0..process_count)
            .map(|index| start_end_process(&format!("proc_{index}")))
            .collect(),
    );

    let linear_start = Instant::now();
    let mut linear_nodes = 0_usize;
    for _ in 0..iterations {
        let process = package
            .find_process(&target_process_id)
            .must("linear wait lookup should find the process");
        linear_nodes += process.nodes.len();
    }
    let linear_elapsed = linear_start.elapsed();

    let indexed_start = Instant::now();
    let mut indexed_nodes = 0_usize;
    for _ in 0..iterations {
        let process =
            indexed_wait_process_lookup(&package, &target_process_id, target_process_index)
                .must("indexed wait lookup should find the process");
        indexed_nodes += process.nodes.len();
    }
    let indexed_elapsed = indexed_start.elapsed();

    let fallback = indexed_wait_process_lookup(&package, &target_process_id, 0)
        .must("stale wait process index should fall back to process id lookup");
    assert_eq!(fallback.key.process_id.as_ref(), target_process_id);
    assert_eq!(linear_nodes, indexed_nodes);
    black_box((linear_nodes, indexed_nodes));
    eprintln!(
        "performance_probe wait_process_lookup processes={} iterations={} linear_ms={:.3} indexed_ms={:.3}",
        process_count,
        iterations,
        linear_elapsed.as_secs_f64() * 1000.0,
        indexed_elapsed.as_secs_f64() * 1000.0
    );
}

#[test]
#[ignore = "performance probe"]
fn performance_probe_adjacency_builder_compares_legacy_bucket_vs_dense_builder() {
    let node_count = 10_000_u32;
    let iterations = 100_u32;
    let nodes = linear_nodes(node_count);
    let edges = linear_edges(node_count);
    let (
        expected_incoming_offsets,
        expected_incoming_order,
        expected_outgoing_offsets,
        expected_outgoing_order,
    ) = dense_build_adjacency_indexes(nodes.len(), &edges);
    let process = BpmnProcessSpec::new(
        ProcessKey::new("pkg_perf_graph", "probe", "digest_probe"),
        nodes.clone(),
        edges.clone(),
        Vec::new(),
    );
    assert_eq!(process.incoming_offsets, expected_incoming_offsets);
    assert_eq!(process.incoming_edge_order, expected_incoming_order);
    assert_eq!(process.outgoing_offsets, expected_outgoing_offsets);
    assert_eq!(process.outgoing_edge_order, expected_outgoing_order);

    let legacy_start = Instant::now();
    for _ in 0..iterations {
        let _ = legacy_build_adjacency_indexes(nodes.len(), &edges);
    }
    let legacy_elapsed = legacy_start.elapsed();

    let dense_start = Instant::now();
    for _ in 0..iterations {
        let _ = dense_build_adjacency_indexes(nodes.len(), &edges);
    }
    let dense_elapsed = dense_start.elapsed();

    eprintln!(
        "performance_probe adjacency_builder nodes={} edges={} iterations={} legacy_ms={:.3} dense_ms={:.3}",
        node_count,
        edges.len(),
        iterations,
        legacy_elapsed.as_secs_f64() * 1000.0,
        dense_elapsed.as_secs_f64() * 1000.0
    );
}

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

    assert_eq!(linear_sum, batch_lookup_sum);
    eprintln!(
        "performance_probe frontier_token_lookup tokens={} lookups_per_batch={} iterations={} linear_ms={:.3} batch_lookup_ms={:.3}",
        token_count,
        lookup_ids.len(),
        iterations,
        linear_elapsed.as_secs_f64() * 1000.0,
        batch_lookup_elapsed.as_secs_f64() * 1000.0
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

#[test]
#[ignore = "performance probe"]
fn performance_probe_runtime_frontier_planning_compares_public_snapshot_vs_direct_proposals() {
    let node_count = 20_000_u32;
    let token_count = 10_000_u64;
    let iterations = 128_u32;
    let process = frontier_probe_process("runtime_frontier_planning_probe", node_count);
    let package = Arc::new(BpmnPackage::new(
        "pkg_runtime_frontier_planning_probe",
        vec![process],
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "runtime_frontier_planning_probe",
        BpmnInstanceInit::new("wf_runtime_frontier_planning_probe", json!({}), 1),
    )
    .must("runtime frontier planning probe instance should be created");
    instance.active_tokens = build_frontier_snapshot_probe_tokens(token_count, node_count);
    for node_state in &mut instance.node_states {
        node_state.status = NodeRuntimeStatus::Idle;
    }
    for token in &instance.active_tokens {
        instance.node_states[token.node_index as usize].status = NodeRuntimeStatus::Queued;
    }
    let process = &package.processes[0];

    let public_start = Instant::now();
    let mut public_proposal_sum = 0_usize;
    let mut public_step_sum = 0_usize;
    for _ in 0..iterations {
        let plan = plan_frontier_step(process, &instance);
        public_proposal_sum += frontier_action_proposal_weight(&plan.action);
        public_step_sum += frontier_action_step_weight(&plan.action);
    }
    let public_elapsed = public_start.elapsed();

    let direct_wrapped_start = Instant::now();
    let mut direct_wrapped_proposal_sum = 0_usize;
    let mut direct_wrapped_step_sum = 0_usize;
    for _ in 0..iterations {
        let proposals = direct_runtime_execution_proposals(&instance);
        let steps = merge_frontier_execution_steps(process, &proposals);
        direct_wrapped_proposal_sum += proposals.len();
        direct_wrapped_step_sum += frontier_step_weight(&steps);
    }
    let direct_wrapped_elapsed = direct_wrapped_start.elapsed();

    let runtime_fast_path_start = Instant::now();
    let mut runtime_fast_path_proposal_sum = 0_usize;
    for _ in 0..iterations {
        let proposals = direct_runtime_execution_proposals(&instance);
        runtime_fast_path_proposal_sum += proposals.len();
    }
    let runtime_fast_path_elapsed = runtime_fast_path_start.elapsed();

    assert_eq!(public_proposal_sum, direct_wrapped_proposal_sum);
    assert_eq!(public_proposal_sum, runtime_fast_path_proposal_sum);
    assert_eq!(public_step_sum, direct_wrapped_step_sum);
    assert_eq!(public_step_sum, public_proposal_sum);
    black_box((
        public_proposal_sum,
        public_step_sum,
        direct_wrapped_proposal_sum,
        direct_wrapped_step_sum,
        runtime_fast_path_proposal_sum,
    ));
    eprintln!(
        "performance_probe runtime_frontier_planning nodes={} tokens={} iterations={} public_snapshot_ms={:.3} direct_wrapped_steps_ms={:.3} runtime_fast_path_ms={:.3}",
        node_count,
        token_count,
        iterations,
        public_elapsed.as_secs_f64() * 1000.0,
        direct_wrapped_elapsed.as_secs_f64() * 1000.0,
        runtime_fast_path_elapsed.as_secs_f64() * 1000.0
    );
}

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

fn indexed_wait_process_lookup<'a>(
    package: &'a BpmnPackage,
    process_id: &str,
    process_index: u32,
) -> Option<&'a BpmnProcessSpec> {
    package
        .processes
        .get(process_index as usize)
        .filter(|process| process.key.process_id.as_ref() == process_id)
        .or_else(|| package.find_process(process_id))
}

fn linear_nodes(node_count: u32) -> Vec<BpmnNodeSpec> {
    (0..node_count)
        .map(|index| match index {
            0 => BpmnNodeSpec::new(index, format!("start_{index}"), BpmnNodeKind::StartEvent),
            i if i == node_count - 1 => {
                BpmnNodeSpec::new(index, format!("end_{index}"), BpmnNodeKind::EndEvent)
            }
            _ => BpmnNodeSpec::new(index, format!("task_{index}"), BpmnNodeKind::ServiceTask),
        })
        .collect()
}

fn linear_edges(node_count: u32) -> Vec<BpmnEdgeSpec> {
    (0..node_count - 1)
        .map(|index| BpmnEdgeSpec::new(index, index + 1, None::<&str>))
        .collect()
}

fn frontier_probe_process(process_id: &str, node_count: u32) -> BpmnProcessSpec {
    BpmnProcessSpec::new(
        ProcessKey::new(
            "pkg_runtime_frontier_planning_probe",
            process_id,
            format!("digest_{process_id}"),
        ),
        linear_nodes(node_count),
        linear_edges(node_count),
        Vec::new(),
    )
}

fn build_event_competition_tokens(wait_count: u32, unrelated_token_count: u32) -> Vec<TokenRecord> {
    let mut active_tokens = Vec::with_capacity((wait_count + unrelated_token_count) as usize);
    active_tokens.extend((0..wait_count).map(|offset| TokenRecord {
        token_id: u64::from(offset) + 1,
        node_index: 2 + offset,
        incoming_edge_index: Some(offset),
        inclusive_join_hint: None,
    }));
    active_tokens.extend((0..unrelated_token_count).map(|offset| TokenRecord {
        token_id: u64::from(wait_count + offset) + 1,
        node_index: 1_000 + offset,
        incoming_edge_index: None,
        inclusive_join_hint: None,
    }));
    active_tokens
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

fn build_frontier_probe_node_statuses(node_count: u32) -> Vec<NodeRuntimeStatus> {
    (0..node_count)
        .map(|node_index| match node_index % 17 {
            0 => NodeRuntimeStatus::Queued,
            1 => NodeRuntimeStatus::Executing,
            2 => NodeRuntimeStatus::Completed,
            3 => NodeRuntimeStatus::Cancelled,
            4 => NodeRuntimeStatus::Failed,
            _ => NodeRuntimeStatus::Idle,
        })
        .collect()
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

fn build_frontier_snapshot_waiting_nodes(wait_count: u32) -> Vec<u32> {
    (0..wait_count).map(|offset| 100 + offset * 3).collect()
}

fn build_frontier_snapshot_boundary_blocking_nodes(wait_count: u32, node_count: u32) -> Vec<u32> {
    (0..wait_count)
        .map(|offset| 5 + ((offset * 37) % (node_count - 5)))
        .collect()
}

fn hashset_frontier_snapshot_classification_sum(
    active_tokens: &[TokenRecord],
    node_statuses: &[NodeRuntimeStatus],
    waiting_node_indices: &[u32],
    boundary_blocking_node_indices: &[u32],
) -> u64 {
    let pending_token_ids = HashSet::<u64>::new();
    let waiting_node_indices: HashSet<u32> = waiting_node_indices.iter().copied().collect();
    let boundary_blocking_node_indices: HashSet<u32> =
        boundary_blocking_node_indices.iter().copied().collect();
    let queued_node_indices: HashSet<u32> = node_statuses
        .iter()
        .enumerate()
        .filter_map(|(node_index, status)| {
            (status == &NodeRuntimeStatus::Queued)
                .then_some(u32::try_from(node_index).must("node index should fit in u32"))
        })
        .collect();
    let terminal_node_statuses = node_statuses
        .iter()
        .map(terminal_frontier_status_for_node)
        .collect::<Vec<_>>();

    active_tokens
        .iter()
        .map(|token| {
            let status = if pending_token_ids.contains(&token.token_id) {
                BpmnFrontierEntryStatus::BlockedOnHost
            } else if waiting_node_indices.contains(&token.node_index) {
                BpmnFrontierEntryStatus::WaitingExternal
            } else if boundary_blocking_node_indices.contains(&token.node_index) {
                if queued_node_indices.contains(&token.node_index) {
                    BpmnFrontierEntryStatus::Runnable
                } else {
                    BpmnFrontierEntryStatus::WaitingExternal
                }
            } else {
                terminal_node_statuses
                    .get(token.node_index as usize)
                    .and_then(|status| *status)
                    .unwrap_or(BpmnFrontierEntryStatus::Runnable)
            };
            frontier_status_code(status)
        })
        .sum()
}

fn dense_frontier_snapshot_classification_sum(
    active_tokens: &[TokenRecord],
    node_statuses: &[NodeRuntimeStatus],
    waiting_node_indices: &[u32],
    boundary_blocking_node_indices: &[u32],
) -> u64 {
    let pending_token_ids = HashSet::<u64>::new();
    let waiting_node_indices: HashSet<u32> = waiting_node_indices.iter().copied().collect();
    let boundary_blocking_node_indices: HashSet<u32> =
        boundary_blocking_node_indices.iter().copied().collect();
    let node_frontier_statuses = node_statuses
        .iter()
        .map(dense_frontier_status_for_node)
        .collect::<Vec<_>>();

    active_tokens
        .iter()
        .map(|token| {
            let status = if pending_token_ids.contains(&token.token_id) {
                BpmnFrontierEntryStatus::BlockedOnHost
            } else if waiting_node_indices.contains(&token.node_index) {
                BpmnFrontierEntryStatus::WaitingExternal
            } else if boundary_blocking_node_indices.contains(&token.node_index) {
                if node_frontier_statuses
                    .get(token.node_index as usize)
                    .and_then(|status| *status)
                    == Some(BpmnFrontierEntryStatus::Runnable)
                {
                    BpmnFrontierEntryStatus::Runnable
                } else {
                    BpmnFrontierEntryStatus::WaitingExternal
                }
            } else {
                node_frontier_statuses
                    .get(token.node_index as usize)
                    .and_then(|status| *status)
                    .unwrap_or(BpmnFrontierEntryStatus::Runnable)
            };
            frontier_status_code(status)
        })
        .sum()
}

fn direct_frontier_snapshot_classification_sum(
    active_tokens: &[TokenRecord],
    node_statuses: &[NodeRuntimeStatus],
    waiting_node_indices: &[u32],
    boundary_blocking_node_indices: &[u32],
) -> u64 {
    let pending_token_ids = HashSet::<u64>::new();
    let mut wait_roles_by_node =
        HashMap::<u32, ProbeWaitRole>::with_capacity(waiting_node_indices.len() * 2);
    for node_index in waiting_node_indices {
        wait_roles_by_node
            .entry(*node_index)
            .or_default()
            .direct_wait = true;
    }
    for node_index in boundary_blocking_node_indices {
        wait_roles_by_node
            .entry(*node_index)
            .or_default()
            .boundary_blocking = true;
    }

    active_tokens
        .iter()
        .map(|token| {
            let status = if pending_token_ids.contains(&token.token_id) {
                BpmnFrontierEntryStatus::BlockedOnHost
            } else if let Some(wait_role) = wait_roles_by_node.get(&token.node_index) {
                if wait_role.direct_wait {
                    BpmnFrontierEntryStatus::WaitingExternal
                } else if wait_role.boundary_blocking {
                    if direct_frontier_status_for_node(node_statuses, token.node_index)
                        == Some(BpmnFrontierEntryStatus::Runnable)
                    {
                        BpmnFrontierEntryStatus::Runnable
                    } else {
                        BpmnFrontierEntryStatus::WaitingExternal
                    }
                } else {
                    direct_frontier_status_for_node(node_statuses, token.node_index)
                        .unwrap_or(BpmnFrontierEntryStatus::Runnable)
                }
            } else {
                direct_frontier_status_for_node(node_statuses, token.node_index)
                    .unwrap_or(BpmnFrontierEntryStatus::Runnable)
            };
            frontier_status_code(status)
        })
        .sum()
}

fn adaptive_frontier_snapshot_classification_sum(
    active_tokens: &[TokenRecord],
    node_statuses: &[NodeRuntimeStatus],
    waiting_node_indices: &[u32],
    boundary_blocking_node_indices: &[u32],
) -> u64 {
    let pending_token_ids = HashSet::<u64>::new();
    let wait_roles = ProbeWaitRoleLookup::new(
        active_tokens.len(),
        node_statuses.len(),
        waiting_node_indices,
        boundary_blocking_node_indices,
    );

    active_tokens
        .iter()
        .map(|token| {
            let status = if pending_token_ids.contains(&token.token_id) {
                BpmnFrontierEntryStatus::BlockedOnHost
            } else if let Some(wait_role) = wait_roles.get(token.node_index) {
                if wait_role.direct_wait {
                    BpmnFrontierEntryStatus::WaitingExternal
                } else if wait_role.boundary_blocking {
                    if direct_frontier_status_for_node(node_statuses, token.node_index)
                        == Some(BpmnFrontierEntryStatus::Runnable)
                    {
                        BpmnFrontierEntryStatus::Runnable
                    } else {
                        BpmnFrontierEntryStatus::WaitingExternal
                    }
                } else {
                    direct_frontier_status_for_node(node_statuses, token.node_index)
                        .unwrap_or(BpmnFrontierEntryStatus::Runnable)
                }
            } else {
                direct_frontier_status_for_node(node_statuses, token.node_index)
                    .unwrap_or(BpmnFrontierEntryStatus::Runnable)
            };
            frontier_status_code(status)
        })
        .sum()
}

fn direct_runtime_execution_proposals(
    instance: &BpmnInstanceState,
) -> Vec<BpmnFrontierExecutionProposal> {
    instance
        .active_tokens
        .iter()
        .enumerate()
        .filter_map(|(token_index, token)| {
            let status = instance
                .node_states
                .get(token.node_index as usize)
                .map(|node_state| &node_state.status);
            (status == Some(&NodeRuntimeStatus::Queued)).then_some(BpmnFrontierExecutionProposal {
                token_id: token.token_id,
                token_index,
                node_index: token.node_index,
                incoming_edge_index: token.incoming_edge_index,
            })
        })
        .collect()
}

fn frontier_action_proposal_weight(action: &BpmnFrontierPlanAction) -> usize {
    match action {
        BpmnFrontierPlanAction::ExecuteBatch(batch) => batch.proposals.len(),
        BpmnFrontierPlanAction::BlockedOnHost(pending) => pending.len(),
        BpmnFrontierPlanAction::WaitingExternalEvent
        | BpmnFrontierPlanAction::Suspended(_)
        | BpmnFrontierPlanAction::Stalled => 0,
    }
}

fn frontier_action_step_weight(action: &BpmnFrontierPlanAction) -> usize {
    match action {
        BpmnFrontierPlanAction::ExecuteBatch(batch) => frontier_step_weight(&batch.steps),
        BpmnFrontierPlanAction::BlockedOnHost(_)
        | BpmnFrontierPlanAction::WaitingExternalEvent
        | BpmnFrontierPlanAction::Suspended(_)
        | BpmnFrontierPlanAction::Stalled => 0,
    }
}

fn frontier_step_weight(steps: &[BpmnFrontierExecutionStep]) -> usize {
    steps
        .iter()
        .map(|step| match step {
            BpmnFrontierExecutionStep::Proposal(_) => 1,
            BpmnFrontierExecutionStep::ParallelJoin(group) => group.proposals.len(),
        })
        .sum()
}

#[derive(Debug, Clone, Copy, Default)]
struct ProbeWaitRole {
    direct_wait: bool,
    boundary_blocking: bool,
}

#[derive(Debug)]
enum ProbeWaitRoleLookup {
    Empty,
    Sparse(HashMap<u32, ProbeWaitRole>),
    Dense(Vec<ProbeWaitRole>),
}

impl ProbeWaitRoleLookup {
    fn new(
        active_token_count: usize,
        node_count: usize,
        waiting_node_indices: &[u32],
        boundary_blocking_node_indices: &[u32],
    ) -> Self {
        if waiting_node_indices.is_empty() && boundary_blocking_node_indices.is_empty() {
            return Self::Empty;
        }

        if should_probe_use_dense_wait_roles(
            active_token_count,
            node_count,
            waiting_node_indices.len() + boundary_blocking_node_indices.len(),
        ) {
            let mut wait_roles = vec![ProbeWaitRole::default(); node_count];
            for node_index in waiting_node_indices {
                if let Some(wait_role) = wait_roles.get_mut(*node_index as usize) {
                    wait_role.direct_wait = true;
                }
            }
            for node_index in boundary_blocking_node_indices {
                if let Some(wait_role) = wait_roles.get_mut(*node_index as usize) {
                    wait_role.boundary_blocking = true;
                }
            }
            return Self::Dense(wait_roles);
        }

        let mut wait_roles: HashMap<u32, ProbeWaitRole> = HashMap::with_capacity(
            waiting_node_indices.len() + boundary_blocking_node_indices.len(),
        );
        for node_index in waiting_node_indices {
            wait_roles.entry(*node_index).or_default().direct_wait = true;
        }
        for node_index in boundary_blocking_node_indices {
            wait_roles.entry(*node_index).or_default().boundary_blocking = true;
        }
        Self::Sparse(wait_roles)
    }

    fn get(&self, node_index: u32) -> Option<ProbeWaitRole> {
        match self {
            Self::Empty => None,
            Self::Sparse(wait_roles) => wait_roles.get(&node_index).copied(),
            Self::Dense(wait_roles) => wait_roles
                .get(node_index as usize)
                .copied()
                .filter(|role| role.is_active()),
        }
    }
}

impl ProbeWaitRole {
    fn is_active(self) -> bool {
        self.direct_wait || self.boundary_blocking
    }
}

fn should_probe_use_dense_wait_roles(
    active_token_count: usize,
    node_count: usize,
    wait_role_count: usize,
) -> bool {
    wait_role_count >= PROBE_DENSE_WAIT_ROLE_THRESHOLD
        && active_token_count >= PROBE_DENSE_WAIT_ROLE_TOKEN_THRESHOLD
        && node_count
            <= active_token_count.saturating_mul(PROBE_DENSE_WAIT_ROLE_NODE_TO_TOKEN_RATIO)
}

fn terminal_frontier_status_for_node(
    status: &NodeRuntimeStatus,
) -> Option<BpmnFrontierEntryStatus> {
    match status {
        NodeRuntimeStatus::Cancelled => Some(BpmnFrontierEntryStatus::Cancelled),
        NodeRuntimeStatus::Failed => Some(BpmnFrontierEntryStatus::Failed),
        NodeRuntimeStatus::Idle
        | NodeRuntimeStatus::Queued
        | NodeRuntimeStatus::Executing
        | NodeRuntimeStatus::Completed => None,
    }
}

fn dense_frontier_status_for_node(status: &NodeRuntimeStatus) -> Option<BpmnFrontierEntryStatus> {
    match status {
        NodeRuntimeStatus::Queued => Some(BpmnFrontierEntryStatus::Runnable),
        NodeRuntimeStatus::Cancelled => Some(BpmnFrontierEntryStatus::Cancelled),
        NodeRuntimeStatus::Failed => Some(BpmnFrontierEntryStatus::Failed),
        NodeRuntimeStatus::Idle | NodeRuntimeStatus::Executing | NodeRuntimeStatus::Completed => {
            None
        }
    }
}

fn direct_frontier_status_for_node(
    node_statuses: &[NodeRuntimeStatus],
    node_index: u32,
) -> Option<BpmnFrontierEntryStatus> {
    node_statuses
        .get(node_index as usize)
        .and_then(dense_frontier_status_for_node)
}

fn frontier_status_code(status: BpmnFrontierEntryStatus) -> u64 {
    match status {
        BpmnFrontierEntryStatus::Runnable => 1,
        BpmnFrontierEntryStatus::BlockedOnHost => 2,
        BpmnFrontierEntryStatus::WaitingExternal => 3,
        BpmnFrontierEntryStatus::Cancelled => 4,
        BpmnFrontierEntryStatus::Failed => 5,
    }
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

fn legacy_build_adjacency_indexes(
    node_count: usize,
    edges: &[BpmnEdgeSpec],
) -> (Vec<BpmnIndexRange>, Vec<u32>, Vec<BpmnIndexRange>, Vec<u32>) {
    let mut incoming_buckets = vec![Vec::new(); node_count];
    let mut outgoing_buckets = vec![Vec::new(); node_count];

    for (edge_index, edge) in edges.iter().enumerate() {
        let edge_index = u32::try_from(edge_index).must("edge index should fit in u32");
        outgoing_buckets[edge.from as usize].push(edge_index);
        incoming_buckets[edge.to as usize].push(edge_index);
    }

    let (incoming_offsets, incoming_edge_order) = flatten_buckets(incoming_buckets);
    let (outgoing_offsets, outgoing_edge_order) = flatten_buckets(outgoing_buckets);

    (
        incoming_offsets,
        incoming_edge_order,
        outgoing_offsets,
        outgoing_edge_order,
    )
}

fn flatten_buckets(buckets: Vec<Vec<u32>>) -> (Vec<BpmnIndexRange>, Vec<u32>) {
    let mut offsets = Vec::with_capacity(buckets.len());
    let mut flattened = Vec::new();

    for bucket in buckets {
        let start = u32::try_from(flattened.len()).must("flattened offsets should fit in u32");
        flattened.extend(bucket);
        let end = u32::try_from(flattened.len()).must("flattened offsets should fit in u32");
        offsets.push(BpmnIndexRange::new(start, end));
    }

    (offsets, flattened)
}

fn dense_build_adjacency_indexes(
    node_count: usize,
    edges: &[BpmnEdgeSpec],
) -> (Vec<BpmnIndexRange>, Vec<u32>, Vec<BpmnIndexRange>, Vec<u32>) {
    let mut incoming_counts = vec![0_u32; node_count];
    let mut outgoing_counts = vec![0_u32; node_count];

    for edge in edges {
        outgoing_counts[edge.from as usize] += 1;
        incoming_counts[edge.to as usize] += 1;
    }

    let incoming_offsets = build_index_ranges(&incoming_counts);
    let outgoing_offsets = build_index_ranges(&outgoing_counts);
    let mut incoming_edge_order = vec![0_u32; edges.len()];
    let mut outgoing_edge_order = vec![0_u32; edges.len()];
    let mut incoming_cursors = incoming_offsets
        .iter()
        .map(|range| range.start)
        .collect::<Vec<_>>();
    let mut outgoing_cursors = outgoing_offsets
        .iter()
        .map(|range| range.start)
        .collect::<Vec<_>>();

    for (edge_index, edge) in edges.iter().enumerate() {
        let edge_index = u32::try_from(edge_index).must("edge index should fit in u32");
        write_edge_index(
            &mut outgoing_cursors,
            &mut outgoing_edge_order,
            edge.from as usize,
            edge_index,
        );
        write_edge_index(
            &mut incoming_cursors,
            &mut incoming_edge_order,
            edge.to as usize,
            edge_index,
        );
    }

    (
        incoming_offsets,
        incoming_edge_order,
        outgoing_offsets,
        outgoing_edge_order,
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

fn build_index_ranges(counts: &[u32]) -> Vec<BpmnIndexRange> {
    let mut offsets = Vec::with_capacity(counts.len());
    let mut start = 0_u32;

    for count in counts {
        let end = start + *count;
        offsets.push(BpmnIndexRange::new(start, end));
        start = end;
    }

    offsets
}

fn write_edge_index(
    cursors: &mut [u32],
    edge_order: &mut [u32],
    node_index: usize,
    edge_index: u32,
) {
    if let Some(cursor) = cursors.get_mut(node_index) {
        edge_order[*cursor as usize] = edge_index;
        *cursor += 1;
    }
}
