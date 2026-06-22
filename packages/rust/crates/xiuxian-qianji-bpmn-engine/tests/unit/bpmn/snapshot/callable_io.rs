use super::snapshot_fixture;
use crate::test_support::MustExt as _;

#[test]
fn bpmn_snapshot_preserves_callable_io_metadata() {
    let snapshot = snapshot_fixture("metadata-callable-io.bpmn");

    let process = snapshot
        .process("Process_CallableIo")
        .must("callable IO process should be indexed by id");
    assert_eq!(process.io_specification_count, 1);
    let process_io = &process.io_specifications[0];
    assert_eq!(
        process_io.io_specification_id.as_deref(),
        Some("ProcessIoSpec_Callable")
    );
    assert_eq!(
        process_io.data_inputs[0].data_id.as_deref(),
        Some("ProcessInput_Request")
    );
    assert_eq!(
        process_io.data_outputs[0].data_id.as_deref(),
        Some("ProcessOutput_Response")
    );
    assert_eq!(process.io_binding_count, 1);
    let process_binding = &process.io_bindings[0];
    assert_eq!(
        process_binding.binding_id.as_deref(),
        Some("ProcessIoBinding_Callable")
    );
    assert_eq!(
        process_binding.operation_ref.as_deref(),
        Some("Operation_Callable")
    );
    assert_eq!(
        process_binding.input_data_ref.as_deref(),
        Some("ProcessInput_Request")
    );
    assert_eq!(
        process_binding.output_data_ref.as_deref(),
        Some("ProcessOutput_Response")
    );

    let global_task = snapshot
        .root
        .global_tasks
        .iter()
        .find(|task| task.task_id.as_deref() == Some("GlobalTask_CallableIo"))
        .must("callable IO global task should be preserved");
    assert_eq!(global_task.supported_interface_refs, ["Interface_Callable"]);
    assert_eq!(global_task.io_specification_count, 1);
    let global_io = &global_task.io_specifications[0];
    assert_eq!(
        global_io.io_specification_id.as_deref(),
        Some("GlobalIoSpec_Callable")
    );
    assert_eq!(
        global_io.data_inputs[0].data_id.as_deref(),
        Some("GlobalInput_Request")
    );
    assert_eq!(
        global_io.data_outputs[0].data_id.as_deref(),
        Some("GlobalOutput_Response")
    );
    assert_eq!(global_task.io_binding_count, 1);
    let global_binding = &global_task.io_bindings[0];
    assert_eq!(
        global_binding.binding_id.as_deref(),
        Some("GlobalIoBinding_Callable")
    );
    assert_eq!(
        global_binding.operation_ref.as_deref(),
        Some("Operation_Callable")
    );
    assert_eq!(
        global_binding.input_data_ref.as_deref(),
        Some("GlobalInput_Request")
    );
    assert_eq!(
        global_binding.output_data_ref.as_deref(),
        Some("GlobalOutput_Response")
    );
}
