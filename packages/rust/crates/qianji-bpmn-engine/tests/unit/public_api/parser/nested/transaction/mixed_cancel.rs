use super::{BpmnEventKind, BpmnParseOptions, fixture_source, parse_bpmn_package};
use crate::test_support::MustExt as _;

#[test]
fn parser_transaction_owner_materializes_external_and_cancel_boundaries_in_source_order() {
    let package = parse_bpmn_package(
        &[fixture_source("transaction-mixed-cancel-boundaries.bpmn")],
        &BpmnParseOptions::default(),
    )
    .must("bounded transaction shell should accept mixed external and cancel boundaries");
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
    assert_eq!(boundaries.len(), 2);
    assert_eq!(boundaries[0].bpmn_id.as_ref(), "tx_timeout");
    assert_eq!(boundaries[1].bpmn_id.as_ref(), "tx_cancel_boundary");

    let timeout_event = process
        .event_for_node(boundaries[0].index)
        .must("timeout boundary event should materialize");
    assert_eq!(timeout_event.kind, BpmnEventKind::Timer);

    let cancel_event = process
        .event_for_node(boundaries[1].index)
        .must("cancel boundary event should materialize");
    assert_eq!(cancel_event.kind, BpmnEventKind::Cancel);
}
