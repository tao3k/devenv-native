use super::super::super::fixture_source;
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{
    BpmnEngineError, BpmnEventKind, BpmnNodeKind, BpmnParseOptions, BpmnSubProcessKind,
    parse_bpmn_package,
};

#[test]
fn parser_call_activity_materializes_called_process_reference() {
    let package = parse_bpmn_package(
        &[fixture_source("call-activity-basic.bpmn")],
        &BpmnParseOptions::default(),
    )
    .must("bounded call activity should parse");
    let process = package
        .find_process("main_process")
        .must("main process should be present");

    assert_eq!(process.nodes[1].kind, BpmnNodeKind::SubProcess);
    assert_eq!(
        process.nodes[1].called_process_id.as_deref(),
        Some("child_process")
    );
    assert!(package.find_process("child_process").is_some());
}

#[test]
fn parser_call_activity_missing_target_is_rejected() {
    let error = parse_bpmn_package(
        &[fixture_source("invalid-call-activity-missing-target.bpmn")],
        &BpmnParseOptions::default(),
    )
    .must_err("call activity should target an existing process");

    assert_eq!(
        error,
        BpmnEngineError::UnknownCalledProcess {
            process_id: "main_process".to_string(),
            node_id: "invoke_child".to_string(),
            called_process_id: "missing_process".to_string(),
        }
    );
}

#[test]
fn parser_call_activity_error_path_materializes_parent_boundaries_and_child_error_end() {
    let package = parse_bpmn_package(
        &[fixture_source("call-activity-error-boundary.bpmn")],
        &BpmnParseOptions::default(),
    )
    .must("bounded call activity error path should parse");
    let main_process = package
        .find_process("main_process")
        .must("main process should be present");
    let child_process = package
        .find_process("child_process")
        .must("child process should be present");

    assert_eq!(main_process.nodes[1].kind, BpmnNodeKind::SubProcess);
    assert_eq!(
        main_process.nodes[1].subprocess_kind,
        Some(BpmnSubProcessKind::CallActivity)
    );
    assert_eq!(
        main_process.nodes[1].called_process_id.as_deref(),
        Some("child_process")
    );
    assert_eq!(child_process.nodes[4].kind, BpmnNodeKind::EndEvent);
    assert_eq!(
        child_process
            .event_for_node(child_process.nodes[4].index)
            .must("child error end should expose event metadata")
            .kind,
        BpmnEventKind::Error
    );
}

#[test]
fn parser_call_activity_error_path_requires_matching_parent_boundary() {
    let error = parse_bpmn_package(
        &[fixture_source(
            "invalid-call-activity-error-missing-boundary.bpmn",
        )],
        &BpmnParseOptions::default(),
    )
    .must_err("every call activity error end should require a matching parent boundary");

    assert_eq!(
        error,
        BpmnEngineError::UnsupportedSubProcessConfiguration {
            process_id: "main_process".to_string(),
            node_id: "invoke_review".to_string(),
            detail: "call_activity_error_missing_boundary",
        }
    );
}

#[test]
fn parser_call_activity_interrupting_external_boundaries_accept_timer_message_signal_and_conditional()
 {
    for fixture in [
        (
            "call-activity-timer-boundary.bpmn",
            "review_timeout",
            BpmnEventKind::Timer,
        ),
        (
            "call-activity-message-boundary.bpmn",
            "review_escalated",
            BpmnEventKind::Message,
        ),
        (
            "call-activity-signal-boundary.bpmn",
            "review_alert",
            BpmnEventKind::Signal,
        ),
        (
            "call-activity-conditional-boundary.bpmn",
            "review_condition",
            BpmnEventKind::Conditional,
        ),
    ] {
        let package =
            parse_bpmn_package(&[fixture_source(fixture.0)], &BpmnParseOptions::default())
                .must("bounded call activity external boundary should parse");
        let main_process = package
            .find_process("main_process")
            .must("main process should be present");

        assert_eq!(main_process.nodes[1].kind, BpmnNodeKind::SubProcess);
        assert_eq!(
            main_process.nodes[1].subprocess_kind,
            Some(BpmnSubProcessKind::CallActivity)
        );
        assert_eq!(
            main_process.nodes[1].called_process_id.as_deref(),
            Some("child_process")
        );
        assert_eq!(
            main_process
                .event_for_node(
                    main_process
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

#[test]
fn parser_call_activity_accepts_mixed_external_and_error_boundaries() {
    let package = parse_bpmn_package(
        &[fixture_source("call-activity-mixed-boundaries.bpmn")],
        &BpmnParseOptions::default(),
    )
    .must("bounded same-package call activity mixed boundary subset should parse");
    let main_process = package
        .find_process("main_process")
        .must("main process should be present");
    let child_process = package
        .find_process("child_process")
        .must("child process should be present");

    assert_eq!(
        main_process.nodes[1].subprocess_kind,
        Some(BpmnSubProcessKind::CallActivity)
    );
    assert_eq!(
        main_process.nodes[1].called_process_id.as_deref(),
        Some("child_process")
    );
    assert_eq!(
        main_process
            .event_for_node(
                main_process
                    .nodes
                    .iter()
                    .find(|node| node.bpmn_id.as_ref() == "review_timeout")
                    .must("timer boundary should exist")
                    .index,
            )
            .must("timer boundary event metadata should exist")
            .kind,
        BpmnEventKind::Timer
    );
    assert_eq!(
        child_process
            .event_for_node(child_process.nodes[4].index)
            .must("child error end should expose event metadata")
            .kind,
        BpmnEventKind::Error
    );
}
