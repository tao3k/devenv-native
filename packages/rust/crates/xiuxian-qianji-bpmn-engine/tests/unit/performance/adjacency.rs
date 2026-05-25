use crate::test_support::MustExt as _;
use std::time::Instant;
use xiuxian_qianji_bpmn_engine::{
    BpmnEdgeSpec, BpmnIndexRange, BpmnNodeKind, BpmnNodeSpec, BpmnProcessSpec, ProcessKey,
};

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
