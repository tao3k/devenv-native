use super::super::fixture_source;
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{
    BpmnCompensationHandlerSpec, BpmnEngineError, BpmnEventKind, BpmnNodeKind, BpmnParseOptions,
    parse_bpmn_package,
};

const TRANSACTION_PROCESS_ID: &str = "__transaction__::main_process::payment_tx";

#[test]
fn parser_transaction_cancel_compensation_materializes_handler_binding() {
    let package = parse_bpmn_package(
        &[fixture_source("transaction-cancel-compensation.bpmn")],
        &BpmnParseOptions::default(),
    )
    .must("bounded transaction compensation should parse");
    let child = package
        .find_process(TRANSACTION_PROCESS_ID)
        .must("transaction shell child process should be present");
    let review = child
        .nodes
        .iter()
        .find(|node| node.bpmn_id.as_ref() == "tx_review")
        .must("compensated activity should be present");
    let handler = child
        .nodes
        .iter()
        .find(|node| node.bpmn_id.as_ref() == "tx_refund")
        .must("compensation handler should be present");
    let boundary = child
        .nodes
        .iter()
        .find(|node| node.bpmn_id.as_ref() == "tx_review_comp_boundary")
        .must("compensation boundary should be present");

    assert_eq!(review.kind, BpmnNodeKind::UserTask);
    assert!(!review.is_for_compensation);
    assert_eq!(handler.kind, BpmnNodeKind::UserTask);
    assert!(handler.is_for_compensation);
    assert_eq!(
        child
            .event_for_node(boundary.index)
            .must("compensation boundary should materialize an event")
            .kind,
        BpmnEventKind::Compensation
    );
    assert_eq!(
        child
            .compensation_handler_for_activity(review.index)
            .must("compensation binding should be indexed"),
        &BpmnCompensationHandlerSpec {
            boundary: boundary.index,
            activity: review.index,
            handler: handler.index,
        }
    );
}

#[test]
fn parser_transaction_compensation_requires_handler_marker() {
    let error = parse_bpmn_package(
        &[fixture_source(
            "invalid-transaction-compensation-missing-marker.bpmn",
        )],
        &BpmnParseOptions::default(),
    )
    .must_err("compensation handler marker should stay explicit");

    assert_eq!(
        error,
        BpmnEngineError::UnsupportedCompensationConfiguration {
            process_id: TRANSACTION_PROCESS_ID.to_string(),
            node_id: "tx_refund".to_string(),
            detail: "missing_compensation_handler_marker",
        }
    );
}

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

#[test]
fn parser_rejects_async_throw_compensation_end_event_with_compensation_detail() {
    let error = parse_bpmn_package(
        &[fixture_source("invalid-throw-compensation-end-async.bpmn")],
        &BpmnParseOptions::default(),
    )
    .must_err("async throw compensation end events should stay deferred");

    assert_eq!(
        error,
        BpmnEngineError::UnsupportedCompensationConfiguration {
            process_id: TRANSACTION_PROCESS_ID.to_string(),
            node_id: "tx_throw_end".to_string(),
            detail: "async_throw_compensation_end_event",
        }
    );
}

#[test]
fn parser_rejects_default_compensation_end_event_with_compensation_detail() {
    let error = parse_bpmn_package(
        &[fixture_source("invalid-default-compensation-end.bpmn")],
        &BpmnParseOptions::default(),
    )
    .must_err("default compensation end events should stay deferred");

    assert_eq!(
        error,
        BpmnEngineError::UnsupportedCompensationConfiguration {
            process_id: "throw_compensation_flow".to_string(),
            node_id: "throw_end".to_string(),
            detail: "default_compensation_end_event",
        }
    );
}

#[test]
fn parser_rejects_async_throw_compensation_intermediate_event_with_compensation_detail() {
    let error = parse_bpmn_package(
        &[fixture_source(
            "invalid-throw-compensation-intermediate-async.bpmn",
        )],
        &BpmnParseOptions::default(),
    )
    .must_err("async throw compensation intermediate events should stay deferred");

    assert_eq!(
        error,
        BpmnEngineError::UnsupportedCompensationConfiguration {
            process_id: TRANSACTION_PROCESS_ID.to_string(),
            node_id: "tx_throw_intermediate".to_string(),
            detail: "async_throw_compensation_intermediate_event",
        }
    );
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
            process_id: "throw_compensation_flow".to_string(),
            node_id: "throw_intermediate".to_string(),
            detail: "throw_compensation_intermediate_event",
        }
    );
}

#[test]
fn parser_rejects_default_compensation_intermediate_event_with_compensation_detail() {
    let error = parse_bpmn_package(
        &[fixture_source(
            "invalid-default-compensation-intermediate.bpmn",
        )],
        &BpmnParseOptions::default(),
    )
    .must_err("default compensation intermediate events should stay deferred");

    assert_eq!(
        error,
        BpmnEngineError::UnsupportedCompensationConfiguration {
            process_id: "throw_compensation_flow".to_string(),
            node_id: "throw_intermediate".to_string(),
            detail: "default_compensation_intermediate_event",
        }
    );
}
