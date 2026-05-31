use arrow::array::{Array, Float64Array, Int32Array, StringArray};
use xiuxian_qianji_control::{
    ActivityId, ActivityTask, ActivityType, ControlEvent, ControlEventRecord, IdempotencyKey,
    RunId, StepId, TaskQueue,
};

pub(super) fn record(sequence: u64, event: ControlEvent) -> ControlEventRecord {
    ControlEventRecord { sequence, event }
}

pub(super) fn run_id() -> RunId {
    RunId::new("bpmn.workflow.run-console-read-model")
        .unwrap_or_else(|error| panic!("run id should be valid: {error}"))
}

pub(super) fn step_id(value: &str) -> StepId {
    StepId::new(value).unwrap_or_else(|error| panic!("step id should be valid: {error}"))
}

pub(super) fn activity_task(
    activity_id: &str,
    activity_type: &str,
    task_queue: &str,
) -> ActivityTask {
    ActivityTask::new(
        ActivityId::new(activity_id)
            .unwrap_or_else(|error| panic!("activity id should be valid: {error}")),
        ActivityType::new(activity_type)
            .unwrap_or_else(|error| panic!("activity type should be valid: {error}")),
        TaskQueue::new(task_queue)
            .unwrap_or_else(|error| panic!("task queue should be valid: {error}")),
        IdempotencyKey::new(format!("{activity_id}-idempotency-key"))
            .unwrap_or_else(|error| panic!("idempotency key should be valid: {error}")),
    )
}

pub(super) fn assert_element_state(
    batch: &arrow::record_batch::RecordBatch,
    element_id: &str,
    state: &str,
    source_event_id: &str,
) {
    let element_ids = string_column(batch, "elementId");
    let row = (0..batch.num_rows())
        .find(|row| element_ids.value(*row) == element_id)
        .unwrap_or_else(|| panic!("element state should include {element_id}"));
    assert_eq!(string_value(batch, "state", row), state);
    assert_eq!(string_value(batch, "sourceEventId", row), source_event_id);
}

pub(super) fn string_value<'a>(
    batch: &'a arrow::record_batch::RecordBatch,
    column: &str,
    row: usize,
) -> &'a str {
    string_column(batch, column).value(row)
}

fn string_column<'a>(batch: &'a arrow::record_batch::RecordBatch, column: &str) -> &'a StringArray {
    batch
        .column_by_name(column)
        .and_then(|array| array.as_any().downcast_ref::<StringArray>())
        .unwrap_or_else(|| panic!("{column} should be a StringArray"))
}

pub(super) fn int32_value(
    batch: &arrow::record_batch::RecordBatch,
    column: &str,
    row: usize,
) -> i32 {
    batch
        .column_by_name(column)
        .and_then(|array| array.as_any().downcast_ref::<Int32Array>())
        .unwrap_or_else(|| panic!("{column} should be an Int32Array"))
        .value(row)
}

pub(super) fn float64_value(
    batch: &arrow::record_batch::RecordBatch,
    column: &str,
    row: usize,
) -> f64 {
    batch
        .column_by_name(column)
        .and_then(|array| array.as_any().downcast_ref::<Float64Array>())
        .unwrap_or_else(|| panic!("{column} should be a Float64Array"))
        .value(row)
}

pub(super) fn assert_float64_eq(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < f64::EPSILON,
        "expected {actual} to equal {expected}"
    );
}
