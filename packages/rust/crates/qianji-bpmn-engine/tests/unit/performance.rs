use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{
    BpmnEdgeSpec, BpmnIndexRange, BpmnInstanceInit, BpmnNodeKind, BpmnNodeSpec, BpmnPackage,
    BpmnProcessSpec, ProcessKey, TokenRecord, create_instance,
};
use serde_json::json;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Instant;

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

    assert_eq!(linear_winner_index_sum, indexed_winner_index_sum);
    assert_eq!(linear_survivor_sum, indexed_survivor_sum);
    eprintln!(
        "performance_probe event_competition_wait_resolution waits={} unrelated_tokens={} iterations={} linear_ms={:.3} indexed_ms={:.3}",
        wait_count,
        unrelated_token_count,
        iterations,
        linear_elapsed.as_secs_f64() * 1000.0,
        indexed_elapsed.as_secs_f64() * 1000.0
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
