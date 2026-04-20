//! BPMN package and process specification types.

use super::{BpmnEdgeSpec, BpmnEventSpec, BpmnIndexRange, BpmnNodeIndex, BpmnNodeSpec};
use crate::dmn::{DmnDecisionDefinition, DmnDecisionRef};
use crate::error::{BpmnEngineError, Result};
use std::sync::Arc;

/// Stable process identity metadata.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ProcessKey {
    /// Package identifier.
    pub package_id: Arc<str>,
    /// Process identifier.
    pub process_id: Arc<str>,
    /// Spec digest or content hash placeholder.
    pub spec_digest_hex: Arc<str>,
}

impl ProcessKey {
    /// Creates a process identity.
    #[must_use]
    pub fn new(
        package_id: impl AsRef<str>,
        process_id: impl AsRef<str>,
        spec_digest_hex: impl AsRef<str>,
    ) -> Self {
        Self {
            package_id: Arc::<str>::from(package_id.as_ref()),
            process_id: Arc::<str>::from(process_id.as_ref()),
            spec_digest_hex: Arc::<str>::from(spec_digest_hex.as_ref()),
        }
    }
}

/// Immutable BPMN process specification.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnProcessSpec {
    /// Stable process identity.
    pub key: ProcessKey,
    /// Process nodes.
    pub nodes: Vec<BpmnNodeSpec>,
    /// Process edges.
    pub edges: Vec<BpmnEdgeSpec>,
    /// Process event bindings.
    pub events: Vec<BpmnEventSpec>,
    /// Dense lookup from node index to event-spec index.
    pub event_index_by_node: Vec<Option<u32>>,
    /// Dense lookup from attached host node index to boundary event node index.
    pub boundary_event_index_by_attached_node: Vec<Option<u32>>,
    /// Precomputed incoming adjacency ranges.
    pub incoming_offsets: Vec<BpmnIndexRange>,
    /// Dense lookup table for incoming edge indices.
    pub incoming_edge_order: Vec<u32>,
    /// Precomputed outgoing adjacency ranges.
    pub outgoing_offsets: Vec<BpmnIndexRange>,
    /// Dense lookup table for outgoing edge indices.
    pub outgoing_edge_order: Vec<u32>,
}

impl BpmnProcessSpec {
    /// Creates a process specification shell.
    #[must_use]
    pub fn new(
        key: ProcessKey,
        nodes: Vec<BpmnNodeSpec>,
        edges: Vec<BpmnEdgeSpec>,
        events: Vec<BpmnEventSpec>,
    ) -> Self {
        let event_index_by_node = build_event_index_lookup(nodes.len(), &events);
        let boundary_event_index_by_attached_node = build_boundary_event_index_lookup(&nodes);
        let (incoming_offsets, incoming_edge_order, outgoing_offsets, outgoing_edge_order) =
            build_adjacency_indexes(nodes.len(), &edges);
        Self {
            key,
            nodes,
            edges,
            events,
            event_index_by_node,
            boundary_event_index_by_attached_node,
            incoming_offsets,
            incoming_edge_order,
            outgoing_offsets,
            outgoing_edge_order,
        }
    }

    /// Returns the ordered incoming edge indices for one node.
    #[must_use]
    pub fn incoming_edge_indices(&self, node_index: BpmnNodeIndex) -> &[u32] {
        let range = self.incoming_offsets[node_index as usize];
        &self.incoming_edge_order[range.start as usize..range.end as usize]
    }

    /// Returns the ordered outgoing edge indices for one node.
    #[must_use]
    pub fn outgoing_edge_indices(&self, node_index: BpmnNodeIndex) -> &[u32] {
        let range = self.outgoing_offsets[node_index as usize];
        &self.outgoing_edge_order[range.start as usize..range.end as usize]
    }

    /// Returns the event binding associated with one node, if present.
    #[must_use]
    pub fn event_for_node(&self, node_index: BpmnNodeIndex) -> Option<&BpmnEventSpec> {
        self.event_index_by_node
            .get(node_index as usize)
            .and_then(|slot| slot.map(|event_index| &self.events[event_index as usize]))
    }

    /// Returns the interrupting boundary event attached to one host node, if present.
    #[must_use]
    pub fn boundary_event_for_attached_node(
        &self,
        node_index: BpmnNodeIndex,
    ) -> Option<&BpmnNodeSpec> {
        self.boundary_event_index_by_attached_node
            .get(node_index as usize)
            .and_then(|slot| slot.map(|boundary_index| &self.nodes[boundary_index as usize]))
    }
}

/// Immutable BPMN package containing one or more process specs.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct BpmnPackage {
    /// Package identifier.
    pub package_id: Arc<str>,
    /// Parsed processes in the package.
    pub processes: Vec<BpmnProcessSpec>,
    /// Optional engine-owned DMN decision registry for local business-rule execution.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dmn_decisions: Vec<DmnDecisionDefinition>,
}

impl BpmnPackage {
    /// Creates a package shell.
    #[must_use]
    pub fn new(package_id: impl AsRef<str>, processes: Vec<BpmnProcessSpec>) -> Self {
        Self {
            package_id: Arc::<str>::from(package_id.as_ref()),
            processes,
            dmn_decisions: Vec::new(),
        }
    }

    /// Attaches engine-owned DMN decision definitions to the package.
    #[must_use]
    pub fn with_dmn_decisions(mut self, dmn_decisions: Vec<DmnDecisionDefinition>) -> Self {
        self.dmn_decisions = dmn_decisions;
        self
    }

    /// Finds a process position and spec by BPMN process identifier.
    #[must_use]
    pub fn find_process_position(&self, process_id: &str) -> Option<(u32, &BpmnProcessSpec)> {
        self.processes
            .iter()
            .enumerate()
            .find_map(|(index, process)| {
                (process.key.process_id.as_ref() == process_id)
                    .then_some((usize_to_u32(index, "process position"), process))
            })
    }

    /// Finds a process by BPMN process identifier.
    #[must_use]
    pub fn find_process(&self, process_id: &str) -> Option<&BpmnProcessSpec> {
        self.find_process_position(process_id)
            .map(|(_, process)| process)
    }

    /// Returns the registered DMN decision definitions owned by the package.
    #[must_use]
    pub fn dmn_decisions(&self) -> &[DmnDecisionDefinition] {
        &self.dmn_decisions
    }

    /// Finds one deterministic DMN decision definition for a business-rule reference.
    ///
    /// # Errors
    ///
    /// Returns [`BpmnEngineError::AmbiguousDmnDecisionReference`] when more
    /// than one registered definition matches the provided reference.
    pub fn find_dmn_decision(
        &self,
        decision_ref: &DmnDecisionRef,
    ) -> Result<Option<&DmnDecisionDefinition>> {
        let mut matches = self
            .dmn_decisions
            .iter()
            .filter(|decision| decision.matches_reference(decision_ref));
        let Some(first_match) = matches.next() else {
            return Ok(None);
        };
        let additional_matches = matches.count();
        if additional_matches > 0 {
            return Err(BpmnEngineError::AmbiguousDmnDecisionReference {
                decision_id: decision_ref.decision_id.to_string(),
                source_id: decision_ref.source_id.as_ref().map(ToString::to_string),
                count: additional_matches + 1,
                source_suffix: decision_ref
                    .source_id
                    .as_ref()
                    .map(|source_id| format!(" in source '{source_id}'"))
                    .unwrap_or_default(),
            });
        }
        Ok(Some(first_match))
    }
}

fn build_adjacency_indexes(
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
        write_edge_index(
            &mut outgoing_cursors,
            &mut outgoing_edge_order,
            edge.from as usize,
            usize_to_u32(edge_index, "outgoing edge index"),
        );
        write_edge_index(
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

fn build_event_index_lookup(node_count: usize, events: &[BpmnEventSpec]) -> Vec<Option<u32>> {
    let mut lookup = vec![None; node_count];
    for (event_index, event) in events.iter().enumerate() {
        if let Some(slot) = lookup.get_mut(event.node_index as usize) {
            *slot = Some(usize_to_u32(event_index, "event index"));
        }
    }
    lookup
}

fn build_boundary_event_index_lookup(nodes: &[BpmnNodeSpec]) -> Vec<Option<u32>> {
    let mut lookup = vec![None; nodes.len()];
    for node in nodes {
        if let Some(attached_to) = node.attached_to {
            lookup[attached_to as usize] = Some(node.index);
        }
    }
    lookup
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

fn usize_to_u32(index: usize, context: &'static str) -> u32 {
    match u32::try_from(index) {
        Ok(value) => value,
        Err(error) => panic!("{context} exceeds u32::MAX: {error}"),
    }
}
