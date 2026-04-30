use super::data::{RawDataObjectReferenceSpec, RawDataObjectSpec};
use super::node::RawNode;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RawPackageDocument {
    pub(crate) source_id: String,
    pub(crate) package_id: String,
    pub(crate) processes: Vec<RawProcess>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RawProcess {
    pub(crate) process_id: String,
    pub(crate) scope: RawProcessScope,
    pub(crate) nodes: Vec<RawNode>,
    pub(crate) flows: Vec<RawSequenceFlow>,
    pub(crate) associations: Vec<RawAssociation>,
    pub(crate) data_objects: Vec<RawDataObjectSpec>,
    pub(crate) data_object_references: Vec<RawDataObjectReferenceSpec>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum RawProcessScope {
    TopLevel,
    NestedShell {
        owner_process_id: String,
        owner_node_id: String,
        kind: NestedShellKind,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NestedShellKind {
    EmbeddedSubProcess,
    Transaction,
    EventSubProcess,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RawSubProcessKind {
    CallActivity,
    EmbeddedSubProcess,
    Transaction,
    EventSubProcess,
}

impl RawProcess {
    pub(in crate::parser::import) fn new_top_level(process_id: String) -> Self {
        Self {
            process_id,
            scope: RawProcessScope::TopLevel,
            nodes: Vec::new(),
            flows: Vec::new(),
            associations: Vec::new(),
            data_objects: Vec::new(),
            data_object_references: Vec::new(),
        }
    }

    pub(in crate::parser::import) fn new_nested_shell(
        process_id: String,
        owner_process_id: String,
        owner_node_id: String,
        kind: NestedShellKind,
    ) -> Self {
        Self {
            process_id,
            scope: RawProcessScope::NestedShell {
                owner_process_id,
                owner_node_id,
                kind,
            },
            nodes: Vec::new(),
            flows: Vec::new(),
            associations: Vec::new(),
            data_objects: Vec::new(),
            data_object_references: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RawSequenceFlow {
    pub(crate) flow_id: String,
    pub(crate) source_ref: String,
    pub(crate) target_ref: String,
    pub(crate) label: Option<String>,
    pub(crate) condition_expression: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RawAssociation {
    pub(crate) association_id: String,
    pub(crate) source_ref: String,
    pub(crate) target_ref: String,
}
