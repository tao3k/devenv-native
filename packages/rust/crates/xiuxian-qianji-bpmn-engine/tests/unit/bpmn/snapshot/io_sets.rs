use super::snapshot_fixture;
use crate::test_support::MustExt as _;

#[test]
fn bpmn_snapshot_preserves_io_set_metadata() {
    let snapshot = snapshot_fixture("metadata-io-sets.bpmn");

    let process = snapshot
        .process("Process_IoSets")
        .must("IO-set process should be indexed by id");
    let process_io = &process.io_specifications[0];
    assert_eq!(
        process_io.io_specification_id.as_deref(),
        Some("ProcessIoSpec_IoSets")
    );
    let process_input_set = &process_io.input_sets[0];
    assert_eq!(
        process_input_set.set_id.as_deref(),
        Some("ProcessInputSet_Main")
    );
    assert_eq!(process_input_set.name.as_deref(), Some("process input set"));
    assert_eq!(process_input_set.data_input_refs, ["ProcessInput_Request"]);
    assert_eq!(
        process_input_set.optional_input_refs,
        ["ProcessInput_Optional"]
    );
    assert_eq!(
        process_input_set.while_executing_input_refs,
        ["ProcessInput_Stream"]
    );
    assert_eq!(process_input_set.output_set_refs, ["ProcessOutputSet_Main"]);

    let process_output_set = &process_io.output_sets[0];
    assert_eq!(
        process_output_set.set_id.as_deref(),
        Some("ProcessOutputSet_Main")
    );
    assert_eq!(
        process_output_set.name.as_deref(),
        Some("process output set")
    );
    assert_eq!(
        process_output_set.data_output_refs,
        ["ProcessOutput_Response"]
    );
    assert_eq!(
        process_output_set.optional_output_refs,
        ["ProcessOutput_Optional"]
    );
    assert_eq!(
        process_output_set.while_executing_output_refs,
        ["ProcessOutput_Stream"]
    );
    assert_eq!(process_output_set.input_set_refs, ["ProcessInputSet_Main"]);

    let global_task = snapshot
        .root
        .global_tasks
        .iter()
        .find(|task| task.task_id.as_deref() == Some("GlobalTask_IoSets"))
        .must("global task should be preserved");
    let global_io = &global_task.io_specifications[0];
    assert_eq!(
        global_io.input_sets[0].data_input_refs,
        ["GlobalInput_Request"]
    );
    assert_eq!(
        global_io.output_sets[0].data_output_refs,
        ["GlobalOutput_Response"]
    );
}
