use super::{
    BpmnEventKind, BpmnNodeKind, BpmnParseOptions, BpmnSubProcessKind, TRANSACTION_PROCESS_ID,
    fixture_source, parse_bpmn_package,
};
use crate::test_support::MustExt as _;

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
fn parser_transaction_interrupting_external_boundaries_accept_timer_message_and_signal() {
    for fixture in [
        (
            "transaction-timer-boundary.bpmn",
            "tx_timeout",
            BpmnEventKind::Timer,
        ),
        (
            "transaction-message-boundary.bpmn",
            "tx_escalated",
            BpmnEventKind::Message,
        ),
        (
            "transaction-signal-boundary.bpmn",
            "tx_alert",
            BpmnEventKind::Signal,
        ),
    ] {
        let package =
            parse_bpmn_package(&[fixture_source(fixture.0)], &BpmnParseOptions::default())
                .must("bounded transaction external boundary should parse");
        let process = package
            .find_process("main_process")
            .must("main process should be present");
        let transaction = process
            .nodes
            .iter()
            .find(|node| node.bpmn_id.as_ref() == "payment_tx")
            .must("transaction shell should be present");

        assert_eq!(transaction.kind, BpmnNodeKind::SubProcess);
        assert_eq!(
            transaction.subprocess_kind,
            Some(BpmnSubProcessKind::Transaction)
        );
        assert_eq!(
            transaction.called_process_id.as_deref(),
            Some(TRANSACTION_PROCESS_ID)
        );
        assert_eq!(
            process
                .event_for_node(
                    process
                        .nodes
                        .iter()
                        .find(|node| node.bpmn_id.as_ref() == fixture.1)
                        .must("boundary node should exist")
                        .index,
                )
                .must("boundary event metadata should exist")
                .kind,
            fixture.2
        );
    }
}
