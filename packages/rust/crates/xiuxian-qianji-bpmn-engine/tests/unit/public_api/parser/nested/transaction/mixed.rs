use super::{BpmnEventKind, BpmnParseOptions, fixture_source, parse_bpmn_package};
use crate::test_support::MustExt as _;

#[test]
fn parser_transaction_owner_materializes_mixed_boundaries_in_source_order() {
    let package = parse_bpmn_package(
        &[fixture_source("transaction-mixed-boundaries.bpmn")],
        &BpmnParseOptions::default(),
    )
    .must("bounded transaction mixed-boundary ownership should parse");
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
    assert_eq!(boundaries[0].bpmn_id.as_ref(), "tx_timeout");
    assert_eq!(boundaries[1].bpmn_id.as_ref(), "tx_error_specific");
    assert_eq!(boundaries[2].bpmn_id.as_ref(), "tx_error_catch_all");

    let timeout_event = process
        .event_for_node(boundaries[0].index)
        .must("timeout boundary event should materialize");
    assert_eq!(timeout_event.kind, BpmnEventKind::Timer);

    let specific_event = process
        .event_for_node(boundaries[1].index)
        .must("specific boundary event should materialize");
    assert_eq!(specific_event.kind, BpmnEventKind::Error);
    assert_eq!(
        specific_event.reference_id.as_deref(),
        Some("payment_error")
    );

    let catch_all_event = process
        .event_for_node(boundaries[2].index)
        .must("catch-all boundary event should materialize");
    assert_eq!(catch_all_event.kind, BpmnEventKind::Error);
    assert_eq!(catch_all_event.reference_id.as_deref(), None);
}
