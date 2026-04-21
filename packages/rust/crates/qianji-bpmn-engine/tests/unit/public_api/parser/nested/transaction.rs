use super::super::super::fixture_source;
use super::TRANSACTION_PROCESS_ID;
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{
    BpmnEngineError, BpmnEventKind, BpmnNodeKind, BpmnParseOptions, BpmnSubProcessKind,
    parse_bpmn_package,
};

#[test]
fn parser_transaction_shell_materializes_synthetic_child_process_reference() {
    let package = parse_bpmn_package(
        &[fixture_source("transaction-basic.bpmn")],
        &BpmnParseOptions::default(),
    )
    .must("bounded transaction shell should parse");
    let process = package
        .find_process("main_process")
        .must("main process should be present");

    assert_eq!(process.nodes[1].kind, BpmnNodeKind::SubProcess);
    assert_eq!(
        process.nodes[1].called_process_id.as_deref(),
        Some(TRANSACTION_PROCESS_ID)
    );

    let child = package
        .find_process(TRANSACTION_PROCESS_ID)
        .must("transaction shell child process should be present");
    assert_eq!(child.nodes.len(), 3);
    assert_eq!(child.nodes[1].kind, BpmnNodeKind::UserTask);
    assert_eq!(child.nodes[1].bpmn_id.as_ref(), "tx_review");
}

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
fn parser_transaction_error_path_materializes_parent_boundary_and_child_error_end() {
    let package = parse_bpmn_package(
        &[fixture_source("transaction-error-boundary.bpmn")],
        &BpmnParseOptions::default(),
    )
    .must("bounded transaction error path should parse");
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
        .find(|node| node.bpmn_id.as_ref() == "tx_error_boundary")
        .must("parent error boundary should be present");

    assert_eq!(transaction.kind, BpmnNodeKind::SubProcess);
    assert_eq!(
        transaction.subprocess_kind,
        Some(BpmnSubProcessKind::Transaction)
    );
    let boundary_event = process
        .event_for_node(boundary.index)
        .must("boundary error event should materialize");
    assert_eq!(boundary_event.kind, BpmnEventKind::Error);
    assert_eq!(
        boundary_event.reference_id.as_deref(),
        Some("payment_error")
    );

    let child = package
        .find_process(TRANSACTION_PROCESS_ID)
        .must("transaction shell child process should be present");
    let error_end = child
        .nodes
        .iter()
        .find(|node| node.bpmn_id.as_ref() == "tx_error_end")
        .must("nested error end should be present");
    let error_event = child
        .event_for_node(error_end.index)
        .must("error end should materialize an error event binding");
    assert_eq!(error_event.kind, BpmnEventKind::Error);
    assert_eq!(error_event.reference_id.as_deref(), Some("payment_error"));
}

#[test]
fn parser_transaction_owner_materializes_multiple_boundaries_in_source_order() {
    let package = parse_bpmn_package(
        &[fixture_source("transaction-multi-error-boundaries.bpmn")],
        &BpmnParseOptions::default(),
    )
    .must("bounded transaction multi-boundary ownership should parse");
    let process = package
        .find_process("main_process")
        .must("main process should be present");
    let transaction = process
        .nodes
        .iter()
        .find(|node| node.bpmn_id.as_ref() == "payment_tx")
        .must("transaction shell should be present");

    let boundaries = process
        .boundary_events_for_attached_node(transaction.index)
        .collect::<Vec<_>>();
    assert_eq!(boundaries.len(), 3);
    assert_eq!(boundaries[0].bpmn_id.as_ref(), "tx_error_specific");
    assert_eq!(boundaries[1].bpmn_id.as_ref(), "tx_error_catch_all");
    assert_eq!(boundaries[2].bpmn_id.as_ref(), "tx_cancel_boundary");

    let specific_event = process
        .event_for_node(boundaries[0].index)
        .must("specific boundary event should materialize");
    assert_eq!(specific_event.kind, BpmnEventKind::Error);
    assert_eq!(
        specific_event.reference_id.as_deref(),
        Some("payment_error")
    );

    let catch_all_event = process
        .event_for_node(boundaries[1].index)
        .must("catch-all boundary event should materialize");
    assert_eq!(catch_all_event.kind, BpmnEventKind::Error);
    assert_eq!(catch_all_event.reference_id.as_deref(), None);

    let cancel_event = process
        .event_for_node(boundaries[2].index)
        .must("cancel boundary event should materialize");
    assert_eq!(cancel_event.kind, BpmnEventKind::Cancel);
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
            process_id: "main_process".to_string(),
            node_id: "tx_cancel_boundary_b".to_string(),
            detail: "multiple_transaction_cancel_boundaries",
        }
    );
}
