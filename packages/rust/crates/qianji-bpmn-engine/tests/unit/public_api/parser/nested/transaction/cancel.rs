use super::{
    BpmnEngineError, BpmnEventKind, BpmnNodeKind, BpmnParseOptions, BpmnSubProcessKind,
    TRANSACTION_PROCESS_ID, fixture_source, parse_bpmn_package,
};
use crate::test_support::MustExt as _;

#[test]
fn parser_transaction_cancel_path_materializes_parent_boundary_and_child_cancel_end() {
    let package = parse_bpmn_package(
        &[fixture_source("transaction-cancel-boundary.bpmn")],
        &BpmnParseOptions::default(),
    )
    .must("bounded transaction cancel path should parse");
    let process = package
        .find_process("main_process")
        .must("main process should be present");
    let transaction = process
        .nodes
        .iter()
        .find(|node| node.bpmn_id.as_ref() == "payment_tx")
        .must("transaction shell should be present");
    let boundary = process
        .nodes
        .iter()
        .find(|node| node.bpmn_id.as_ref() == "tx_cancel_boundary")
        .must("parent cancel boundary should be present");

    assert_eq!(transaction.kind, BpmnNodeKind::SubProcess);
    assert_eq!(
        transaction.subprocess_kind,
        Some(BpmnSubProcessKind::Transaction)
    );
    assert_eq!(
        process
            .event_for_node(boundary.index)
            .must("boundary cancel event should materialize")
            .kind,
        BpmnEventKind::Cancel
    );

    let child = package
        .find_process(TRANSACTION_PROCESS_ID)
        .must("transaction shell child process should be present");
    let cancel_end = child
        .nodes
        .iter()
        .find(|node| node.bpmn_id.as_ref() == "tx_cancel_end")
        .must("nested cancel end should be present");
    assert_eq!(
        child
            .event_for_node(cancel_end.index)
            .must("cancel end should materialize a cancel event binding")
            .kind,
        BpmnEventKind::Cancel
    );
}

#[test]
fn parser_transaction_owner_rejects_multiple_cancel_boundaries() {
    let error = parse_bpmn_package(
        &[fixture_source(
            "invalid-transaction-multiple-cancel-boundaries.bpmn",
        )],
        &BpmnParseOptions::default(),
    )
    .must_err("transaction owner should reject multiple cancel boundaries");

    assert_eq!(
        error,
        BpmnEngineError::UnsupportedBoundaryEventConfiguration {
            process_id: ("main_process".to_string()).into(),
            node_id: ("tx_cancel_boundary_b".to_string()).into(),
            detail: "multiple_transaction_cancel_boundaries",
        }
    );
}
