use super::{TRANSACTION_PROCESS_ID, fixture_source};
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{
    BpmnCompensationHandlerSpec, BpmnEngineError, BpmnEventKind, BpmnNodeKind, BpmnParseOptions,
    parse_bpmn_package,
};

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
