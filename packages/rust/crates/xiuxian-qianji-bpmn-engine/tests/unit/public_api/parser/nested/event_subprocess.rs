use crate::public_api::fixture_source;
use crate::test_support::MustExt as _;
use xiuxian_qianji_bpmn_engine::{
    BpmnEngineError, BpmnEventKind, BpmnNodeKind, BpmnParseOptions, BpmnSubProcessKind,
    BpmnTimerKind, parse_bpmn_package,
};

#[test]
fn parser_event_subprocess_message_materializes_trigger_owner_and_body() {
    let package = parse_bpmn_package(
        &[fixture_source("event-subprocess-message.bpmn")],
        &BpmnParseOptions::default(),
    )
    .must("event subprocess message trigger should parse");
    let process = package
        .find_process("event_subprocess_message")
        .must("parent process should exist");
    let owner = process
        .nodes
        .iter()
        .find(|node| node.bpmn_id.as_ref() == "interrupting_event_subprocess")
        .must("event subprocess owner should be normalized");
    assert_eq!(
        owner.subprocess_kind,
        Some(BpmnSubProcessKind::EventSubProcess)
    );
    assert_eq!(
        owner.called_process_id.as_deref(),
        Some("__event_subprocess__::event_subprocess_message::interrupting_event_subprocess")
    );
    let event = process
        .event_for_node(owner.index)
        .must("event subprocess owner should expose trigger metadata");
    assert_eq!(event.kind, BpmnEventKind::Message);
    assert_eq!(event.reference_id.as_deref(), Some("interrupt_request"));

    let child = package
        .find_process(
            "__event_subprocess__::event_subprocess_message::interrupting_event_subprocess",
        )
        .must("event subprocess body should be materialized");
    assert_eq!(child.nodes[0].kind, BpmnNodeKind::StartEvent);
    assert_eq!(child.nodes[1].kind, BpmnNodeKind::EndEvent);
}

#[test]
fn parser_event_subprocess_accepts_signal_timer_and_conditional_triggers() {
    let cases = [
        (
            "event-subprocess-signal.bpmn",
            "event_subprocess_signal",
            BpmnEventKind::Signal,
        ),
        (
            "event-subprocess-timer.bpmn",
            "event_subprocess_timer",
            BpmnEventKind::Timer,
        ),
        (
            "event-subprocess-conditional.bpmn",
            "event_subprocess_conditional",
            BpmnEventKind::Conditional,
        ),
    ];

    for (fixture, process_id, expected_kind) in cases {
        let package = parse_bpmn_package(&[fixture_source(fixture)], &BpmnParseOptions::default())
            .must("event subprocess trigger fixture should parse");
        let process = package
            .find_process(process_id)
            .must("parent process should exist");
        let owner = process
            .nodes
            .iter()
            .find(|node| node.bpmn_id.as_ref() == "interrupting_event_subprocess")
            .must("event subprocess owner should exist");
        let event = process
            .event_for_node(owner.index)
            .must("owner trigger should be indexed");
        assert_eq!(event.kind, expected_kind);
        if event.kind == BpmnEventKind::Timer {
            assert_eq!(
                event
                    .timer
                    .as_ref()
                    .must("timer event subprocess should preserve timer")
                    .kind,
                BpmnTimerKind::Duration
            );
        }
        if event.kind == BpmnEventKind::Conditional {
            assert_eq!(event.condition_expression.as_deref(), Some("approved"));
        }
    }
}

#[test]
fn parser_event_subprocess_rejects_non_interrupting_start() {
    let error = parse_bpmn_package(
        &[fixture_source(
            "invalid-event-subprocess-non-interrupting.bpmn",
        )],
        &BpmnParseOptions::default(),
    )
    .must_err("non-interrupting event subprocesses should stay deferred");

    assert_eq!(
        error,
        BpmnEngineError::UnsupportedSubProcessConfiguration {
            process_id: ("event_subprocess_non_interrupting".to_string()).into(),
            node_id: ("non_interrupting_event_subprocess".to_string()).into(),
            detail: "event_subprocess_non_interrupting",
        }
    );
}

#[test]
fn parser_event_subprocess_rejects_compensation_trigger() {
    let error = parse_bpmn_package(
        &[fixture_source("invalid-compensation-event-subprocess.bpmn")],
        &BpmnParseOptions::default(),
    )
    .must_err("compensation event subprocesses should stay deferred");

    assert_eq!(
        error,
        BpmnEngineError::UnsupportedSubProcessConfiguration {
            process_id: ("main_process".to_string()).into(),
            node_id: ("comp_handler".to_string()).into(),
            detail: "event_subprocess_compensation_deferred",
        }
    );
}
