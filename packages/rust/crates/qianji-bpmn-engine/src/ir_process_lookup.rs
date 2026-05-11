use crate::ir_edge_api::BpmnEdgeSpec;
use crate::ir_event_api::BpmnEventSpec;
use crate::ir_index_api::BpmnIndexRange;
use crate::ir_node_api::BpmnNodeSpec;

pub(super) fn build_adjacency_indexes(
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
        write_index(
            &mut outgoing_cursors,
            &mut outgoing_edge_order,
            edge.from as usize,
            usize_to_u32(edge_index, "outgoing edge index"),
        );
        write_index(
            &mut incoming_cursors,
            &mut incoming_edge_order,
            edge.to as usize,
            usize_to_u32(edge_index, "incoming edge index"),
        );
    }

    (
        incoming_offsets,
        incoming_edge_order,
        outgoing_offsets,
        outgoing_edge_order,
    )
}

pub(super) fn build_event_index_lookup(
    node_count: usize,
    events: &[BpmnEventSpec],
) -> Vec<Option<u32>> {
    let mut lookup = vec![None; node_count];
    for (event_index, event) in events.iter().enumerate() {
        if let Some(slot) = lookup.get_mut(event.node_index as usize) {
            *slot = Some(usize_to_u32(event_index, "event index"));
        }
    }
    lookup
}

pub(super) fn build_compensation_handler_lookup<T>(
    node_count: usize,
    compensation_handlers: &[T],
    activity_node_index: impl Fn(&T) -> u32,
) -> Vec<Option<u32>> {
    let mut lookup = vec![None; node_count];
    for (index, binding) in compensation_handlers.iter().enumerate() {
        lookup[activity_node_index(binding) as usize] =
            Some(usize_to_u32(index, "compensation handler index"));
    }
    lookup
}

pub(super) fn build_boundary_event_lookup(
    nodes: &[BpmnNodeSpec],
) -> (Vec<BpmnIndexRange>, Vec<u32>) {
    let mut counts = vec![0_u32; nodes.len()];
    nodes
        .iter()
        .filter_map(|node| node.attached_to)
        .for_each(|attached_to| counts[attached_to as usize] += 1);

    let offsets = build_index_ranges(&counts);
    let mut order = vec![0_u32; counts.iter().copied().map(|count| count as usize).sum()];
    let mut cursors = offsets.iter().map(|range| range.start).collect::<Vec<_>>();

    nodes
        .iter()
        .filter_map(|node| {
            node.attached_to
                .map(|attached_to| (attached_to, node.index))
        })
        .for_each(|(attached_to, node_index)| {
            write_index(&mut cursors, &mut order, attached_to as usize, node_index);
        });

    (offsets, order)
}

pub(super) fn build_index_ranges(counts: &[u32]) -> Vec<BpmnIndexRange> {
    let mut offsets = Vec::with_capacity(counts.len());
    let mut start = 0_u32;

    for count in counts {
        let end = start + *count;
        offsets.push(BpmnIndexRange::new(start, end));
        start = end;
    }

    offsets
}

pub(super) fn usize_to_u32(index: usize, context: &'static str) -> u32 {
    match u32::try_from(index) {
        Ok(value) => value,
        Err(error) => panic!("{context} exceeds u32::MAX: {error}"),
    }
}

fn write_index(cursors: &mut [u32], order: &mut [u32], node_index: usize, value: u32) {
    if let Some(cursor) = cursors.get_mut(node_index) {
        order[*cursor as usize] = value;
        *cursor += 1;
    }
}
