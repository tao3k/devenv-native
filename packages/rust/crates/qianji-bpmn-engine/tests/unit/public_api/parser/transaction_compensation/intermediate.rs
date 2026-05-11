use super::{TRANSACTION_PROCESS_ID, fixture_source};
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{
    BpmnEngineError, BpmnEventKind, BpmnNodeKind, BpmnParseOptions, parse_bpmn_package,
};

#[test]
fn parser_transaction_throw_compensation_intermediate_materializes_target_reference() {
    let package = parse_bpmn_package(
        &[fixture_source("transaction-throw-compensation-intermediate.bpmn")],
        &BpmnParseOptions::default(),
    )
    .must(
        "targeted throw compensation intermediate event should parse in the bounded transaction subset",
    );
    let child = package
        .find_process(TRANSACTION_PROCESS_ID)
        .must("transaction shell child process should be present");
    let throw_intermediate = child
        .nodes
        .iter()
        .find(|node| node.bpmn_id.as_ref() == "tx_throw_intermediate")
        .must("throw compensation intermediate event should be present");

    assert_eq!(
        throw_intermediate.kind,
        BpmnNodeKind::IntermediateThrowEvent
    );
    let event = child
        .event_for_node(throw_intermediate.index)
        .must("throw compensation intermediate event should materialize an event");
    assert_eq!(event.kind, BpmnEventKind::Compensation);
    assert_eq!(event.reference_id.as_deref(), Some("tx_review"));
    assert!(event.wait_for_completion);
}

#[test]
fn parser_transaction_default_throw_compensation_intermediate_materializes_unqualified_event() {
    let package = parse_bpmn_package(
        &[fixture_source("transaction-default-compensation-intermediate.bpmn")],
        &BpmnParseOptions::default(),
    )
    .must(
        "default throw compensation intermediate event should parse in the bounded transaction subset",
    );
    let child = package
        .find_process(TRANSACTION_PROCESS_ID)
        .must("transaction shell child process should be present");
    let throw_intermediate = child
        .nodes
        .iter()
        .find(|node| node.bpmn_id.as_ref() == "tx_throw_intermediate_default")
        .must("default throw compensation intermediate event should be present");

    assert_eq!(
        throw_intermediate.kind,
        BpmnNodeKind::IntermediateThrowEvent
    );
    let event = child
        .event_for_node(throw_intermediate.index)
        .must("default throw compensation intermediate event should materialize an event");
    assert_eq!(event.kind, BpmnEventKind::Compensation);
    assert_eq!(event.reference_id, None);
    assert!(event.wait_for_completion);
}

#[test]
fn parser_transaction_async_throw_compensation_intermediate_materializes_target_reference() {
    let package = parse_bpmn_package(
        &[fixture_source("transaction-throw-compensation-intermediate-async.bpmn")],
        &BpmnParseOptions::default(),
    )
    .must(
        "async throw compensation intermediate event should parse in the bounded transaction subset",
    );
    let child = package
        .find_process(TRANSACTION_PROCESS_ID)
        .must("transaction shell child process should be present");
    let throw_intermediate = child
        .nodes
        .iter()
        .find(|node| node.bpmn_id.as_ref() == "tx_throw_intermediate_async")
        .must("async throw compensation intermediate event should be present");

    assert_eq!(
        throw_intermediate.kind,
        BpmnNodeKind::IntermediateThrowEvent
    );
    let event = child
        .event_for_node(throw_intermediate.index)
        .must("async throw compensation intermediate event should materialize an event");
    assert_eq!(event.kind, BpmnEventKind::Compensation);
    assert_eq!(event.reference_id.as_deref(), Some("tx_review"));
    assert!(!event.wait_for_completion);
}

#[test]
fn parser_transaction_async_default_throw_compensation_intermediate_materializes_unqualified_event()
{
    let package = parse_bpmn_package(
        &[fixture_source(
            "transaction-default-compensation-intermediate-async.bpmn",
        )],
        &BpmnParseOptions::default(),
    )
    .must(
        "async default throw compensation intermediate event should parse in the bounded transaction subset",
    );
    let child = package
        .find_process(TRANSACTION_PROCESS_ID)
        .must("transaction shell child process should be present");
    let throw_intermediate = child
        .nodes
        .iter()
        .find(|node| node.bpmn_id.as_ref() == "tx_throw_intermediate_default_async")
        .must("async default throw compensation intermediate event should be present");

    assert_eq!(
        throw_intermediate.kind,
        BpmnNodeKind::IntermediateThrowEvent
    );
    let event = child
        .event_for_node(throw_intermediate.index)
        .must("async default throw compensation intermediate event should materialize an event");
    assert_eq!(event.kind, BpmnEventKind::Compensation);
    assert_eq!(event.reference_id, None);
    assert!(!event.wait_for_completion);
}

#[test]
fn parser_rejects_throw_compensation_intermediate_event_with_compensation_detail() {
    let error = parse_bpmn_package(
        &[fixture_source(
            "invalid-throw-compensation-intermediate.bpmn",
        )],
        &BpmnParseOptions::default(),
    )
    .must_err("throw compensation intermediate events should stay deferred");

    assert_eq!(
        error,
        BpmnEngineError::UnsupportedCompensationConfiguration {
            process_id: ("throw_compensation_flow".to_string()).into(),
            node_id: ("throw_intermediate".to_string()).into(),
            detail: "throw_compensation_intermediate_event",
        }
    );
}
