use super::{parse_fixture_error, parse_fixture_package};
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{BpmnEngineError, BpmnNodeKind, BpmnRepeatSpec};

#[test]
fn parser_standard_loop_materializes_repeat_snapshot() {
    let package = parse_fixture_package("standard-loop-service-task.bpmn");
    let process = package
        .find_process("loop_service")
        .must("process should be present");

    assert_eq!(process.nodes[1].kind, BpmnNodeKind::ServiceTask);
    let repeat = process.nodes[1]
        .repeat
        .as_ref()
        .must("standard loop should be materialized");
    match repeat {
        BpmnRepeatSpec::StandardLoop(loop_spec) => {
            assert!(loop_spec.test_before);
            assert_eq!(loop_spec.loop_maximum, Some(3));
            assert_eq!(loop_spec.loop_condition.as_deref(), Some("not done"));
        }
        other @ BpmnRepeatSpec::SequentialMultiInstance(_) => {
            panic!("unexpected repeat snapshot: {other:?}");
        }
    }
}

#[test]
fn parser_standard_loop_requires_maximum_or_condition() {
    let error = parse_fixture_error(
        "invalid-standard-loop-missing-limit.bpmn",
        "standard loop must declare a loop maximum or condition",
    );

    assert_eq!(
        error,
        BpmnEngineError::UnsupportedLoopConfiguration {
            process_id: "loop_invalid".to_string(),
            node_id: "review".to_string(),
            detail: "missing_loop_maximum_or_condition",
        }
    );
}

#[test]
fn parser_sequential_multi_instance_materializes_repeat_snapshot() {
    let package = parse_fixture_package("sequential-multi-instance-service-task.bpmn");
    let process = package
        .find_process("multi_instance_service")
        .must("process should be present");

    assert_eq!(process.nodes[1].kind, BpmnNodeKind::ServiceTask);
    let repeat = process.nodes[1]
        .repeat
        .as_ref()
        .must("sequential multi-instance should be materialized");
    match repeat {
        BpmnRepeatSpec::SequentialMultiInstance(loop_spec) => {
            assert_eq!(loop_spec.loop_cardinality, 3);
        }
        other @ BpmnRepeatSpec::StandardLoop(_) => {
            panic!("unexpected repeat snapshot: {other:?}");
        }
    }
}

#[test]
fn parser_parallel_multi_instance_is_deferred_explicitly() {
    let error = parse_fixture_error(
        "invalid-multi-instance-deferred.bpmn",
        "parallel multi-instance should stay deferred in this bounded slice",
    );

    assert_eq!(
        error,
        BpmnEngineError::UnsupportedLoopConfiguration {
            process_id: "multi_instance_invalid".to_string(),
            node_id: "review".to_string(),
            detail: "parallel_multi_instance_deferred",
        }
    );
}

#[test]
fn parser_sequential_multi_instance_requires_loop_cardinality() {
    let error = parse_fixture_error(
        "invalid-sequential-multi-instance-missing-cardinality.bpmn",
        "sequential multi-instance must declare loopCardinality in this slice",
    );

    assert_eq!(
        error,
        BpmnEngineError::UnsupportedLoopConfiguration {
            process_id: "multi_instance_missing_cardinality".to_string(),
            node_id: "review".to_string(),
            detail: "missing_loop_cardinality",
        }
    );
}
