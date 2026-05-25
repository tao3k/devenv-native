use super::fixture_source;
use crate::test_support::MustExt as _;
use xiuxian_qianji_bpmn_engine::{BpmnDocumentSnapshot, snapshot_bpmn_source};

mod artifacts;
mod callable_io;
mod choreography;
mod collaboration;
mod conversation;
mod data_associations;
mod data_states;
mod diagram;
mod flow_elements;
mod io_sets;
mod partners;
mod process;
mod resource_roles;
mod root_catalogs;
mod typed_catalogs;
mod xml_errors;

fn metadata_snapshot() -> BpmnDocumentSnapshot {
    snapshot_bpmn_source(&fixture_source("metadata-collaboration-lane-data.bpmn"))
        .must("metadata-only BPMN source should still produce a document snapshot")
}

fn snapshot_fixture(name: &str) -> BpmnDocumentSnapshot {
    snapshot_bpmn_source(&fixture_source(name))
        .must("metadata-only BPMN source should produce a document snapshot")
}
