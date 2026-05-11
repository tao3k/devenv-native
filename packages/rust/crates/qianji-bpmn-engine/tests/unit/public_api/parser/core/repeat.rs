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
        other @ (BpmnRepeatSpec::SequentialMultiInstance(_)
        | BpmnRepeatSpec::ParallelMultiInstance(_)) => {
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
            process_id: ("loop_invalid".to_string()).into(),
            node_id: ("review".to_string()).into(),
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
            assert_eq!(loop_spec.loop_cardinality, Some(3));
            assert!(loop_spec.data_binding.is_none());
        }
        other @ (BpmnRepeatSpec::StandardLoop(_) | BpmnRepeatSpec::ParallelMultiInstance(_)) => {
            panic!("unexpected repeat snapshot: {other:?}");
        }
    }
}

#[test]
fn parser_sequential_multi_instance_completion_condition_materializes_repeat_snapshot() {
    let package = parse_fixture_package("sequential-multi-instance-completion-condition.bpmn");
    let process = package
        .find_process("multi_instance_completion_condition_sequential")
        .must("process should be present");

    let repeat = process.nodes[1]
        .repeat
        .as_ref()
        .must("sequential multi-instance should be materialized");
    match repeat {
        BpmnRepeatSpec::SequentialMultiInstance(loop_spec) => {
            assert_eq!(loop_spec.loop_cardinality, Some(5));
            assert!(loop_spec.data_binding.is_none());
            assert_eq!(
                loop_spec.completion_condition.as_deref(),
                Some("completed >= 2")
            );
        }
        other => panic!("unexpected repeat snapshot: {other:?}"),
    }
}

#[test]
fn parser_sequential_multi_instance_data_binding_materializes_repeat_snapshot() {
    let package = parse_fixture_package("sequential-multi-instance-loop-input.bpmn");
    let process = package
        .find_process("multi_instance_loop_input_sequential")
        .must("process should be present");

    let repeat = process.nodes[1]
        .repeat
        .as_ref()
        .must("sequential multi-instance should be materialized");
    match repeat {
        BpmnRepeatSpec::SequentialMultiInstance(loop_spec) => {
            assert_eq!(loop_spec.loop_cardinality, None);
            let data_binding = loop_spec
                .data_binding
                .as_ref()
                .must("data binding should be materialized");
            assert_eq!(data_binding.loop_data_input_ref.as_ref(), "input_data");
            assert_eq!(data_binding.input_data_item.as_ref(), "input_item");
            assert_eq!(
                data_binding.loop_data_output_ref.as_deref(),
                Some("output_data")
            );
            assert_eq!(
                data_binding.output_data_item.as_deref(),
                Some("output_item")
            );
        }
        other => panic!("unexpected repeat snapshot: {other:?}"),
    }
}

#[test]
fn parser_parallel_multi_instance_materializes_repeat_snapshot() {
    let package = parse_fixture_package("parallel-multi-instance-service-task.bpmn");
    let process = package
        .find_process("parallel_multi_instance_service")
        .must("process should be present");

    assert_eq!(process.nodes[1].kind, BpmnNodeKind::ServiceTask);
    let repeat = process.nodes[1]
        .repeat
        .as_ref()
        .must("parallel multi-instance should be materialized");
    match repeat {
        BpmnRepeatSpec::ParallelMultiInstance(loop_spec) => {
            assert_eq!(loop_spec.loop_cardinality, Some(3));
            assert!(loop_spec.data_binding.is_none());
        }
        other => panic!("unexpected repeat snapshot: {other:?}"),
    }
}

#[test]
fn parser_parallel_multi_instance_completion_condition_materializes_repeat_snapshot() {
    let package = parse_fixture_package("parallel-multi-instance-completion-condition.bpmn");
    let process = package
        .find_process("multi_instance_completion_condition_parallel")
        .must("process should be present");

    let repeat = process.nodes[1]
        .repeat
        .as_ref()
        .must("parallel multi-instance should be materialized");
    match repeat {
        BpmnRepeatSpec::ParallelMultiInstance(loop_spec) => {
            assert_eq!(loop_spec.loop_cardinality, Some(3));
            assert!(loop_spec.data_binding.is_none());
            assert_eq!(
                loop_spec.completion_condition.as_deref(),
                Some("completed >= 1")
            );
        }
        other => panic!("unexpected repeat snapshot: {other:?}"),
    }
}

#[test]
fn parser_parallel_multi_instance_data_binding_materializes_repeat_snapshot() {
    let package = parse_fixture_package("parallel-multi-instance-loop-input.bpmn");
    let process = package
        .find_process("multi_instance_loop_input_parallel")
        .must("process should be present");

    let repeat = process.nodes[1]
        .repeat
        .as_ref()
        .must("parallel multi-instance should be materialized");
    match repeat {
        BpmnRepeatSpec::ParallelMultiInstance(loop_spec) => {
            assert_eq!(loop_spec.loop_cardinality, None);
            let data_binding = loop_spec
                .data_binding
                .as_ref()
                .must("data binding should be materialized");
            assert_eq!(data_binding.loop_data_input_ref.as_ref(), "input_data");
            assert_eq!(data_binding.input_data_item.as_ref(), "input_item");
            assert_eq!(
                data_binding.loop_data_output_ref.as_deref(),
                Some("output_data")
            );
            assert_eq!(
                data_binding.output_data_item.as_deref(),
                Some("output_item")
            );
        }
        other => panic!("unexpected repeat snapshot: {other:?}"),
    }
}

#[test]
fn parser_parallel_multi_instance_rejects_unsupported_completion_condition_expression() {
    let error = parse_fixture_error(
        "invalid-multi-instance-deferred.bpmn",
        "parallel multi-instance completion condition should reject unsupported expressions",
    );

    assert_eq!(
        error,
        BpmnEngineError::UnsupportedLoopConfiguration {
            process_id: ("multi_instance_invalid".to_string()).into(),
            node_id: ("review".to_string()).into(),
            detail: "unsupported_multi_instance_completion_condition_expression",
        }
    );
}

#[test]
fn parser_sequential_multi_instance_requires_loop_cardinality() {
    let error = parse_fixture_error(
        "invalid-sequential-multi-instance-missing-cardinality.bpmn",
        "sequential multi-instance must declare either loopCardinality or a collection input in this slice",
    );

    assert_eq!(
        error,
        BpmnEngineError::UnsupportedLoopConfiguration {
            process_id: ("multi_instance_missing_cardinality".to_string()).into(),
            node_id: ("review".to_string()).into(),
            detail: "missing_loop_cardinality_or_data_input",
        }
    );
}

#[test]
fn parser_data_bound_multi_instance_rejects_in_place_output_binding() {
    let error = parse_fixture_error(
        "invalid-sequential-multi-instance-in-place-output.bpmn",
        "data-bound multi-instance output must not overwrite the input collection in place",
    );

    assert_eq!(
        error,
        BpmnEngineError::UnsupportedLoopConfiguration {
            process_id: ("multi_instance_in_place_output".to_string()).into(),
            node_id: ("review".to_string()).into(),
            detail: "unsupported_multi_instance_in_place_output",
        }
    );
}
