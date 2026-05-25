use xiuxian_qianji_bpmn_engine::{
    BpmnCallableBindingExecutionPolicy, BpmnCallableKind, BpmnParseOptions, parse_bpmn_package,
};

use super::fixture_source;
use crate::test_support::MustExt as _;

#[test]
fn callable_registry_materializes_process_and_global_task_io_metadata() {
    let package = parse_bpmn_package(
        &[fixture_source("metadata-callable-io.bpmn")],
        &BpmnParseOptions::default(),
    )
    .must("callable IO fixture should parse");

    let registry = package.callable_registry();
    let process = registry
        .find_definition("Process_CallableIo")
        .must("process callable should be registered");
    assert_eq!(process.kind, BpmnCallableKind::Process);
    assert_eq!(process.is_executable, Some(false));
    assert!(process.runtime_available);
    assert_eq!(
        process.inputs[0].data_id.as_deref(),
        Some("ProcessInput_Request")
    );
    assert_eq!(
        process.outputs[0].data_id.as_deref(),
        Some("ProcessOutput_Response")
    );
    assert_eq!(
        process.io_bindings[0].operation_ref.as_deref(),
        Some("Operation_Callable")
    );

    let global_task = package
        .find_callable_definition("GlobalTask_CallableIo")
        .must("global task callable should be registered");
    assert_eq!(global_task.kind, BpmnCallableKind::GlobalTask);
    assert!(!global_task.runtime_available);
    assert_eq!(
        global_task.supported_interface_refs[0].as_ref(),
        "Interface_Callable"
    );
    assert_eq!(
        global_task.inputs[0].data_id.as_deref(),
        Some("GlobalInput_Request")
    );
    assert_eq!(
        global_task.outputs[0].data_id.as_deref(),
        Some("GlobalOutput_Response")
    );
    assert_eq!(
        global_task.io_bindings[0].input_data_ref.as_deref(),
        Some("GlobalInput_Request")
    );
}

#[test]
fn callable_registry_records_existing_process_target_call_activity_binding() {
    let package = parse_bpmn_package(
        &[fixture_source("call-activity-basic.bpmn")],
        &BpmnParseOptions::default(),
    )
    .must("call activity fixture should parse");

    let bindings = package.call_activity_bindings();
    assert_eq!(bindings.len(), 1);
    let binding = &bindings[0];
    assert_eq!(binding.process_id.as_ref(), "main_process");
    assert_eq!(binding.activity_id.as_ref(), "invoke_child");
    assert_eq!(binding.target_id.as_ref(), "child_process");
    assert_eq!(binding.target_kind, BpmnCallableKind::Process);
    assert_eq!(
        binding.execution_policy,
        BpmnCallableBindingExecutionPolicy::BoundedProcessCall
    );

    let child = package
        .find_callable_definition("child_process")
        .must("called process should be registered as callable");
    assert!(child.runtime_available);
}
