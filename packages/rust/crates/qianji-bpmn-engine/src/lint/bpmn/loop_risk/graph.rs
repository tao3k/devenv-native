//! Canonical `api` entry for loop-risk graph analysis.

use super::{
    BpmnNodeKind, BpmnProcessSpec, DefaultReentryFlow, HashMap, HashSet, ProcessMetadata,
    is_host_task, outgoing_edge_indices,
};

mod api;
mod cycle;
mod default_flow;
mod scc;
mod source;

use source::{incoming_edge_counts, source_component_entry_candidate};

pub(super) use api::{
    component_has_exit_path, default_reentry_flows, is_cyclic_component,
    strongly_connected_components,
};
