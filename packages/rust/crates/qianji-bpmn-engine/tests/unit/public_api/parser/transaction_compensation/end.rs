use super::{TRANSACTION_PROCESS_ID, fixture_source};
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{
    BpmnEngineError, BpmnEventKind, BpmnNodeKind, BpmnParseOptions, parse_bpmn_package,
};

#[test]
fn parser_transaction_throw_compensation_end_materializes_target_reference() {
    let package = parse_bpmn_package(
        &[fixture_source("transaction-throw-compensation-end.bpmn")],
        &BpmnParseOptions::default(),
    )
    .must("targeted throw compensation end event should parse in the bounded transaction subset");
    let child = package
        .find_process(TRANSACTION_PROCESS_ID)
        .must("transaction shell child process should be present");
    let throw_end = child
        .nodes
        .iter()
        .find(|node| node.bpmn_id.as_ref() == "tx_throw_end")
        .must("throw compensation end event should be present");

    assert_eq!(throw_end.kind, BpmnNodeKind::EndEvent);
    let event = child
        .event_for_node(throw_end.index)
        .must("throw compensation end event should materialize an event");
    assert_eq!(event.kind, BpmnEventKind::Compensation);
    assert_eq!(event.reference_id.as_deref(), Some("tx_review"));
}

#[test]
fn parser_transaction_default_throw_compensation_end_materializes_unqualified_event() {
    let package = parse_bpmn_package(
        &[fixture_source("transaction-default-compensation-end.bpmn")],
        &BpmnParseOptions::default(),
    )
    .must("default throw compensation end event should parse in the bounded transaction subset");
    let child = package
        .find_process(TRANSACTION_PROCESS_ID)
        .must("transaction shell child process should be present");
    let throw_end = child
        .nodes
        .iter()
        .find(|node| node.bpmn_id.as_ref() == "tx_throw_end_default")
        .must("default throw compensation end event should be present");

    assert_eq!(throw_end.kind, BpmnNodeKind::EndEvent);
    let event = child
        .event_for_node(throw_end.index)
        .must("default throw compensation end event should materialize an event");
    assert_eq!(event.kind, BpmnEventKind::Compensation);
    assert_eq!(event.reference_id, None);
}

#[test]
fn parser_transaction_async_throw_compensation_end_materializes_target_reference() {
    let package = parse_bpmn_package(
        &[fixture_source(
            "transaction-throw-compensation-end-async.bpmn",
        )],
        &BpmnParseOptions::default(),
    )
    .must("async targeted throw compensation end event should parse in the bounded subset");
    let child = package
        .find_process(TRANSACTION_PROCESS_ID)
        .must("transaction shell child process should be present");
    let throw_end = child
        .nodes
        .iter()
        .find(|node| node.bpmn_id.as_ref() == "tx_throw_end")
        .must("async throw compensation end event should be present");

    assert_eq!(throw_end.kind, BpmnNodeKind::EndEvent);
    let event = child
        .event_for_node(throw_end.index)
        .must("async throw compensation end event should materialize an event");
    assert_eq!(event.kind, BpmnEventKind::Compensation);
    assert_eq!(event.reference_id.as_deref(), Some("tx_review"));
    assert!(!event.wait_for_completion);
}

#[test]
fn parser_transaction_async_default_throw_compensation_end_materializes_unqualified_event() {
    let package = parse_bpmn_package(
        &[fixture_source(
            "transaction-default-compensation-end-async.bpmn",
        )],
        &BpmnParseOptions::default(),
    )
    .must("async default throw compensation end event should parse in the bounded subset");
    let child = package
        .find_process(TRANSACTION_PROCESS_ID)
        .must("transaction shell child process should be present");
    let throw_end = child
        .nodes
        .iter()
        .find(|node| node.bpmn_id.as_ref() == "tx_throw_end_default")
        .must("async default throw compensation end event should be present");

    assert_eq!(throw_end.kind, BpmnNodeKind::EndEvent);
    let event = child
        .event_for_node(throw_end.index)
        .must("async default throw compensation end event should materialize an event");
    assert_eq!(event.kind, BpmnEventKind::Compensation);
    assert_eq!(event.reference_id, None);
    assert!(!event.wait_for_completion);
}

#[test]
fn parser_rejects_throw_compensation_end_event_with_compensation_detail() {
    let error = parse_bpmn_package(
        &[fixture_source("invalid-throw-compensation-end.bpmn")],
        &BpmnParseOptions::default(),
    )
    .must_err(
        "throw compensation end events outside the bounded transaction subset should stay deferred",
    );

    assert_eq!(
        error,
        BpmnEngineError::UnsupportedCompensationConfiguration {
            process_id: "throw_compensation_flow".to_string(),
            node_id: "throw_end".to_string(),
            detail: "throw_compensation_end_event",
        }
    );
}
