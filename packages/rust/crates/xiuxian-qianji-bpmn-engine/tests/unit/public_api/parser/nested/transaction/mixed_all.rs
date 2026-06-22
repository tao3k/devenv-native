use super::{BpmnEngineError, BpmnEventKind, BpmnParseOptions, fixture_source, parse_bpmn_package};
use crate::test_support::MustExt as _;

#[test]
fn parser_transaction_owner_materializes_external_cancel_and_error_boundaries_in_source_order() {
    let package = parse_bpmn_package(
        &[fixture_source(
            "transaction-mixed-cancel-error-boundaries.bpmn",
        )],
        &BpmnParseOptions::default(),
    )
    .must("bounded transaction shell should accept mixed external, cancel, and error boundaries");
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
    assert_eq!(boundaries.len(), 4);
    assert_eq!(boundaries[0].bpmn_id.as_ref(), "tx_timeout");
    assert_eq!(boundaries[1].bpmn_id.as_ref(), "tx_cancel_boundary");
    assert_eq!(boundaries[2].bpmn_id.as_ref(), "tx_error_specific");
    assert_eq!(boundaries[3].bpmn_id.as_ref(), "tx_error_catch_all");

    let timeout_event = process
        .event_for_node(boundaries[0].index)
        .must("timeout boundary event should materialize");
    assert_eq!(timeout_event.kind, BpmnEventKind::Timer);

    let cancel_event = process
        .event_for_node(boundaries[1].index)
        .must("cancel boundary event should materialize");
    assert_eq!(cancel_event.kind, BpmnEventKind::Cancel);

    let specific_event = process
        .event_for_node(boundaries[2].index)
        .must("specific error boundary event should materialize");
    assert_eq!(specific_event.kind, BpmnEventKind::Error);
    assert_eq!(
        specific_event.reference_id.as_deref(),
        Some("payment_error")
    );

    let catch_all_event = process
        .event_for_node(boundaries[3].index)
        .must("catch-all error boundary event should materialize");
    assert_eq!(catch_all_event.kind, BpmnEventKind::Error);
    assert_eq!(catch_all_event.reference_id.as_deref(), None);
}

#[test]
fn parser_transaction_shell_rejects_multiple_external_boundaries_even_with_cancel_and_error() {
    let error = parse_bpmn_package(
        &[fixture_source(
            "invalid-transaction-multiple-external-cancel-error-boundaries.bpmn",
        )],
        &BpmnParseOptions::default(),
    )
    .must_err("bounded transaction shell should still reject more than one external boundary");

    assert_eq!(
        error,
        BpmnEngineError::UnsupportedBoundaryEventConfiguration {
            process_id: ("main_process".to_string()).into(),
            node_id: ("tx_timeout_late".to_string()).into(),
            detail: "multiple_boundary_events_for_attached_node",
        }
    );
}
