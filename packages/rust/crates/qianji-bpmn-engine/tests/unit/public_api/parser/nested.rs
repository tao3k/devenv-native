use super::super::fixture_source;
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{
    BpmnEngineError, BpmnEventKind, BpmnNodeKind, BpmnParseOptions, BpmnSubProcessKind,
    parse_bpmn_package,
};

const EMBEDDED_REVIEW_PROCESS_ID: &str = "__embedded_subprocess__::main_process::inline_review";
const TRANSACTION_PROCESS_ID: &str = "__transaction__::main_process::payment_tx";

#[test]
fn parser_call_activity_materializes_called_process_reference() {
    let package = parse_bpmn_package(
        &[fixture_source("call-activity-basic.bpmn")],
        &BpmnParseOptions::default(),
    )
    .must("bounded call activity should parse");
    let process = package
        .find_process("main_process")
        .must("main process should be present");

    assert_eq!(process.nodes[1].kind, BpmnNodeKind::SubProcess);
    assert_eq!(
        process.nodes[1].called_process_id.as_deref(),
        Some("child_process")
    );
    assert!(package.find_process("child_process").is_some());
}

#[test]
fn parser_call_activity_missing_target_is_rejected() {
    let error = parse_bpmn_package(
        &[fixture_source("invalid-call-activity-missing-target.bpmn")],
        &BpmnParseOptions::default(),
    )
    .must_err("call activity should target an existing process");

    assert_eq!(
        error,
        BpmnEngineError::UnknownCalledProcess {
            process_id: "main_process".to_string(),
            node_id: "invoke_child".to_string(),
            called_process_id: "missing_process".to_string(),
        }
    );
}

#[test]
fn parser_embedded_subprocess_materializes_synthetic_child_process_reference() {
    let package = parse_bpmn_package(
        &[fixture_source("embedded-subprocess-basic.bpmn")],
        &BpmnParseOptions::default(),
    )
    .must("bounded embedded subprocess should parse");
    let process = package
        .find_process("main_process")
        .must("main process should be present");

    assert_eq!(process.nodes[1].kind, BpmnNodeKind::SubProcess);
    assert_eq!(
        process.nodes[1].called_process_id.as_deref(),
        Some(EMBEDDED_REVIEW_PROCESS_ID)
    );

    let child = package
        .find_process(EMBEDDED_REVIEW_PROCESS_ID)
        .must("embedded subprocess child process should be present");
    assert_eq!(child.nodes.len(), 3);
    assert_eq!(child.nodes[1].kind, BpmnNodeKind::UserTask);
    assert_eq!(child.nodes[1].bpmn_id.as_ref(), "sub_review");
}

#[test]
fn parser_embedded_subprocess_requires_exactly_one_start_event() {
    let error = parse_bpmn_package(
        &[fixture_source(
            "invalid-embedded-subprocess-multiple-starts.bpmn",
        )],
        &BpmnParseOptions::default(),
    )
    .must_err("embedded subprocess should reject multiple nested start events");

    assert_eq!(
        error,
        BpmnEngineError::UnsupportedSubProcessConfiguration {
            process_id: "main_process".to_string(),
            node_id: "inline_review".to_string(),
            detail: "embedded_subprocess_start_event_count",
        }
    );
}

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
