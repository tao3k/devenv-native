use crate::ir_data_api::BpmnDataObjectBindingSpec;
use crate::ir_edge_api::BpmnEdgeSpec;
use crate::ir_event_api::BpmnEventSpec;
use crate::ir_index_api::{BpmnIndexRange, BpmnNodeIndex};
use crate::ir_node_api::BpmnNodeSpec;
use crate::ir_process_compensation::BpmnCompensationHandlerSpec;
use crate::ir_process_key::ProcessKey;
use crate::ir_process_lookup::{
    build_adjacency_indexes, build_boundary_event_lookup, build_compensation_handler_lookup,
    build_event_index_lookup,
};

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
    /// Bounded compensation handler bindings.
    #[serde(default)]
    pub compensation_handlers: Vec<BpmnCompensationHandlerSpec>,
    /// Bounded executable process-level BPMN data-object bindings.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub data_object_bindings: Vec<BpmnDataObjectBindingSpec>,
    /// Dense lookup from node index to event-spec index.
    pub event_index_by_node: Vec<Option<u32>>,
    /// Dense lookup from activity node index to compensation-handler binding index.
    pub compensation_handler_index_by_activity: Vec<Option<u32>>,
    /// Dense lookup ranges from attached host node index to boundary-event node indices.
    pub boundary_event_offsets: Vec<BpmnIndexRange>,
    /// Dense lookup table for boundary-event node indices attached to one host node.
    pub boundary_event_order: Vec<u32>,
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
        Self::new_with_compensation(key, nodes, edges, events, Vec::new())
    }

    /// Creates a process specification shell with bounded compensation bindings.
    #[must_use]
    pub fn new_with_compensation(
        key: ProcessKey,
        nodes: Vec<BpmnNodeSpec>,
        edges: Vec<BpmnEdgeSpec>,
        events: Vec<BpmnEventSpec>,
        compensation_handlers: Vec<BpmnCompensationHandlerSpec>,
    ) -> Self {
        let event_index_by_node = build_event_index_lookup(nodes.len(), &events);
        let compensation_handler_index_by_activity =
            build_compensation_handler_lookup(nodes.len(), &compensation_handlers, |binding| {
                binding.activity
            });
        let (boundary_event_offsets, boundary_event_order) = build_boundary_event_lookup(&nodes);
        let (incoming_offsets, incoming_edge_order, outgoing_offsets, outgoing_edge_order) =
            build_adjacency_indexes(nodes.len(), &edges);
        Self {
            key,
            nodes,
            edges,
            events,
            compensation_handlers,
            data_object_bindings: Vec::new(),
            event_index_by_node,
            compensation_handler_index_by_activity,
            boundary_event_offsets,
            boundary_event_order,
            incoming_offsets,
            incoming_edge_order,
            outgoing_offsets,
            outgoing_edge_order,
        }
    }

    /// Attaches bounded executable process-level data-object bindings.
    #[must_use]
    pub fn with_data_object_bindings(
        mut self,
        data_object_bindings: Vec<BpmnDataObjectBindingSpec>,
    ) -> Self {
        self.data_object_bindings = data_object_bindings;
        self
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

    /// Returns the bounded compensation binding registered for one activity.
    #[must_use]
    pub fn compensation_handler_for_activity(
        &self,
        node_index: BpmnNodeIndex,
    ) -> Option<&BpmnCompensationHandlerSpec> {
        self.compensation_handler_index_by_activity
            .get(node_index as usize)
            .and_then(|slot| {
                slot.map(|binding_index| &self.compensation_handlers[binding_index as usize])
            })
    }

    /// Returns the ordered boundary-event node indices attached to one host node.
    #[must_use]
    pub fn boundary_event_indices_for_attached_node(&self, node_index: BpmnNodeIndex) -> &[u32] {
        let range = self.boundary_event_offsets[node_index as usize];
        &self.boundary_event_order[range.start as usize..range.end as usize]
    }

    /// Returns the interrupting boundary event attached to one host node, if present.
    #[must_use]
    pub fn boundary_event_for_attached_node(
        &self,
        node_index: BpmnNodeIndex,
    ) -> Option<&BpmnNodeSpec> {
        self.boundary_event_indices_for_attached_node(node_index)
            .first()
            .map(|boundary_index| &self.nodes[*boundary_index as usize])
    }

    /// Returns the ordered interrupting boundary events attached to one host node.
    pub fn boundary_events_for_attached_node(
        &self,
        node_index: BpmnNodeIndex,
    ) -> impl Iterator<Item = &BpmnNodeSpec> + '_ {
        self.boundary_event_indices_for_attached_node(node_index)
            .iter()
            .map(|boundary_index| &self.nodes[*boundary_index as usize])
    }
}
