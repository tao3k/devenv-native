use super::super::parse_fixture_package;
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{BpmnEventKind, BpmnNodeKind, BpmnRepeatSpec, BpmnTimerKind};

#[test]
fn parser_parallel_multi_instance_non_interrupting_boundary_timer_materializes_repeat_and_attachment()
 {
    let package = parse_fixture_package(
        "boundary-timer-non-interrupt-parallel-mi.bpmn",
        "bounded parallel multi-instance non-interrupting boundary timer should parse",
    );
    let process = package
        .find_process("review_with_parallel_timeout")
        .must("process should be present");

    let repeat = process.nodes[1]
        .repeat
        .as_ref()
        .must("parallel multi-instance repeat should be materialized");
    match repeat {
        BpmnRepeatSpec::ParallelMultiInstance(loop_spec) => {
            assert_eq!(loop_spec.loop_cardinality, Some(3));
            assert!(loop_spec.data_binding.is_none());
        }
        other => panic!("unexpected repeat snapshot: {other:?}"),
    }

    assert_eq!(process.nodes[2].kind, BpmnNodeKind::BoundaryEvent);
    assert_eq!(process.nodes[2].attached_to, Some(1));
    assert!(!process.nodes[2].cancel_activity);
    let boundary = process
        .boundary_event_for_attached_node(1)
        .must("attached task should resolve the boundary event");
    assert_eq!(boundary.index, 2);
    let event = process
        .event_for_node(boundary.index)
        .must("parallel multi-instance boundary timer should materialize an event binding");
    assert_eq!(event.kind, BpmnEventKind::Timer);
    assert_eq!(event.name.as_deref(), Some("review_timeout"));
    let timer = event.timer.as_ref().must("timer snapshot should exist");
    assert_eq!(timer.kind, BpmnTimerKind::Duration);
    assert_eq!(timer.expression.as_ref(), "PT30M");
}

#[test]
fn parser_sequential_multi_instance_non_interrupting_boundary_timer_materializes_repeat_and_attachment()
 {
    let package = parse_fixture_package(
        "boundary-timer-non-interrupt-sequential-mi.bpmn",
        "bounded sequential multi-instance non-interrupting boundary timer should parse",
    );
    let process = package
        .find_process("review_with_sequential_timeout")
        .must("process should be present");

    let repeat = process.nodes[1]
        .repeat
        .as_ref()
        .must("sequential multi-instance repeat should be materialized");
    match repeat {
        BpmnRepeatSpec::SequentialMultiInstance(loop_spec) => {
            assert_eq!(loop_spec.loop_cardinality, Some(3));
            assert!(loop_spec.data_binding.is_none());
        }
        other => panic!("unexpected repeat snapshot: {other:?}"),
    }

    assert_eq!(process.nodes[2].kind, BpmnNodeKind::BoundaryEvent);
    assert_eq!(process.nodes[2].attached_to, Some(1));
    assert!(!process.nodes[2].cancel_activity);
    let boundary = process
        .boundary_event_for_attached_node(1)
        .must("attached task should resolve the boundary event");
    assert_eq!(boundary.index, 2);
    let event = process
        .event_for_node(boundary.index)
        .must("sequential multi-instance boundary timer should materialize an event binding");
    assert_eq!(event.kind, BpmnEventKind::Timer);
    assert_eq!(event.name.as_deref(), Some("review_timeout"));
    let timer = event.timer.as_ref().must("timer snapshot should exist");
    assert_eq!(timer.kind, BpmnTimerKind::Duration);
    assert_eq!(timer.expression.as_ref(), "PT30M");
}

#[test]
fn parser_standard_loop_non_interrupting_boundary_timer_materializes_repeat_and_attachment() {
    let package = parse_fixture_package(
        "boundary-timer-non-interrupt-standard-loop.bpmn",
        "bounded standard-loop non-interrupting boundary timer should parse",
    );
    let process = package
        .find_process("review_with_retries")
        .must("process should be present");

    let repeat = process.nodes[1]
        .repeat
        .as_ref()
        .must("standard-loop repeat should be materialized");
    match repeat {
        BpmnRepeatSpec::StandardLoop(loop_spec) => {
            assert!(loop_spec.test_before);
            assert_eq!(loop_spec.loop_maximum, Some(3));
            assert_eq!(loop_spec.loop_condition.as_deref(), Some("not done"));
        }
        other => panic!("unexpected repeat snapshot: {other:?}"),
    }

    assert_eq!(process.nodes[2].kind, BpmnNodeKind::BoundaryEvent);
    assert_eq!(process.nodes[2].attached_to, Some(1));
    assert!(!process.nodes[2].cancel_activity);
    let boundary = process
        .boundary_event_for_attached_node(1)
        .must("attached task should resolve the boundary event");
    assert_eq!(boundary.index, 2);
    let event = process
        .event_for_node(boundary.index)
        .must("standard-loop boundary timer should materialize an event binding");
    assert_eq!(event.kind, BpmnEventKind::Timer);
    let timer = event.timer.as_ref().must("timer snapshot should exist");
    assert_eq!(timer.kind, BpmnTimerKind::Duration);
    assert_eq!(timer.expression.as_ref(), "PT30M");
}
