use super::{
    BpmnEngineError, BpmnEventKind, BpmnParseOptions, TRANSACTION_PROCESS_ID, fixture_source,
    parse_bpmn_package,
};
use crate::test_support::MustExt as _;

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
    let boundary = process
        .nodes
        .iter()
        .find(|node| node.bpmn_id.as_ref() == "tx_error_boundary")
        .must("parent error boundary should be present");

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
fn parser_transaction_shell_accepts_multiple_nested_error_end_events() {
    let package = parse_bpmn_package(
        &[fixture_source("transaction-multi-error-ends.bpmn")],
        &BpmnParseOptions::default(),
    )
    .must("bounded transaction shell should accept multiple nested error ends");
    let child = package
        .find_process(TRANSACTION_PROCESS_ID)
        .must("transaction shell child process should be present");

    let payment_error_end = child
        .nodes
        .iter()
        .find(|node| node.bpmn_id.as_ref() == "tx_payment_error_end")
        .must("payment error end should be present");
    let fraud_error_end = child
        .nodes
        .iter()
        .find(|node| node.bpmn_id.as_ref() == "tx_fraud_error_end")
        .must("fraud error end should be present");

    assert_eq!(
        child
            .event_for_node(payment_error_end.index)
            .must("payment error end should materialize an event")
            .reference_id
            .as_deref(),
        Some("payment_error")
    );
    assert_eq!(
        child
            .event_for_node(fraud_error_end.index)
            .must("fraud error end should materialize an event")
            .reference_id
            .as_deref(),
        Some("fraud_error")
    );
}

#[test]
fn parser_transaction_shell_rejects_multi_error_end_when_one_error_lacks_boundary() {
    let error = parse_bpmn_package(
        &[fixture_source(
            "invalid-transaction-multi-error-end-missing-boundary.bpmn",
        )],
        &BpmnParseOptions::default(),
    )
    .must_err("every nested error end should still require a matching parent boundary");

    assert_eq!(
        error,
        BpmnEngineError::UnsupportedTransactionConfiguration {
            process_id: "main_process".to_string(),
            node_id: "payment_tx".to_string(),
            detail: "transaction_error_missing_boundary",
        }
    );
}
