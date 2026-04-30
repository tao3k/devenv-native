use super::{EMBEDDED_REVIEW_PROCESS_ID, fixture_source};
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{BpmnEventKind, BpmnNodeKind, BpmnParseOptions, parse_bpmn_package};

#[test]
fn parser_embedded_subprocess_interrupting_external_boundaries_accept_timer_message_signal_and_conditional()
 {
    for (fixture_name, boundary_id, event_kind) in [
        (
            "embedded-subprocess-timer-boundary.bpmn",
            "review_timeout",
            BpmnEventKind::Timer,
        ),
        (
            "embedded-subprocess-message-boundary.bpmn",
            "review_escalated",
            BpmnEventKind::Message,
        ),
        (
            "embedded-subprocess-signal-boundary.bpmn",
            "review_alert",
            BpmnEventKind::Signal,
        ),
        (
            "embedded-subprocess-conditional-boundary.bpmn",
            "review_condition",
            BpmnEventKind::Conditional,
        ),
    ] {
        let package = parse_bpmn_package(
            &[fixture_source(fixture_name)],
            &BpmnParseOptions::default(),
        )
        .must("embedded subprocess interrupting external boundary should parse");
        let process = package
            .find_process("main_process")
            .must("main process should be present");

        assert_eq!(process.nodes[1].kind, BpmnNodeKind::SubProcess);
        assert_eq!(
            process.nodes[1].called_process_id.as_deref(),
            Some(EMBEDDED_REVIEW_PROCESS_ID)
        );
        let boundary = process
            .nodes
            .iter()
            .find(|node| node.bpmn_id.as_ref() == boundary_id)
            .must("embedded subprocess boundary should be present");
        assert_eq!(boundary.kind, BpmnNodeKind::BoundaryEvent);
        assert!(boundary.cancel_activity);
        assert_eq!(
            process
                .event_for_node(boundary.index)
                .must("embedded subprocess boundary should expose event metadata")
                .kind,
            event_kind
        );
    }
}
