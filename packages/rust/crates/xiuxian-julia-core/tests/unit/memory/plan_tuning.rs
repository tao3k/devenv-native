use super::{
    MEMORY_JULIA_PLAN_TUNING_REQUEST_COLUMNS, MemoryJuliaPlanTuningAdviceRow,
    MemoryJuliaPlanTuningRequestRow, build_memory_julia_plan_tuning_request_batch,
    decode_memory_julia_plan_tuning_advice_rows, memory_julia_plan_tuning_response_schema,
    validate_memory_julia_plan_tuning_response_batch,
};
use arrow::array::{Float32Array, StringArray, UInt32Array};
use arrow::record_batch::RecordBatch;
use std::sync::Arc;

fn sample_request_row() -> MemoryJuliaPlanTuningRequestRow {
    MemoryJuliaPlanTuningRequestRow {
        scope: "repo".to_string(),
        scenario_pack: Some("searchinfra".to_string()),
        current_k1: 8,
        current_k2: 4,
        current_lambda: 0.7,
        current_min_score: 0.18,
        current_max_context_chars: 960,
        feedback_bias: -0.4,
        recent_success_rate: 0.35,
        recent_failure_rate: 0.45,
        recent_latency_ms: 210,
    }
}

fn sample_advice_row() -> MemoryJuliaPlanTuningAdviceRow {
    MemoryJuliaPlanTuningAdviceRow {
        scope: "repo".to_string(),
        next_k1: 10,
        next_k2: 5,
        next_lambda: 0.64,
        next_min_score: 0.15,
        next_max_context_chars: 1_120,
        tuning_reason: "negative feedback bias widened recall budget".to_string(),
        confidence: 0.84,
        schema_version: "v1".to_string(),
    }
}

#[test]
fn build_memory_julia_plan_tuning_request_batch_accepts_valid_rows() {
    let batch = build_memory_julia_plan_tuning_request_batch(&[sample_request_row()])
        .unwrap_or_else(|error| panic!("request batch should build: {error}"));
    assert_eq!(batch.num_rows(), 1);
    assert_eq!(
        batch
            .schema()
            .fields()
            .iter()
            .map(|field| field.name().clone())
            .collect::<Vec<_>>(),
        MEMORY_JULIA_PLAN_TUNING_REQUEST_COLUMNS
            .iter()
            .map(std::string::ToString::to_string)
            .collect::<Vec<_>>()
    );
    assert!(batch.column_by_name("feedback_bias").is_some());
}

#[test]
fn build_memory_julia_plan_tuning_request_batch_rejects_invalid_shape() {
    let mut row = sample_request_row();
    row.current_k2 = 9;
    let Err(error) = build_memory_julia_plan_tuning_request_batch(&[row]) else {
        panic!("current_k2 above current_k1 should fail");
    };
    assert!(
        error
            .to_string()
            .contains("current_k2 greater than current_k1")
    );
}

#[test]
fn decode_memory_julia_plan_tuning_advice_rows_decodes_valid_batches() {
    let expected = sample_advice_row();
    let batch = RecordBatch::try_new(
        memory_julia_plan_tuning_response_schema(),
        vec![
            Arc::new(StringArray::from(vec![expected.scope.as_str()])),
            Arc::new(UInt32Array::from(vec![expected.next_k1])),
            Arc::new(UInt32Array::from(vec![expected.next_k2])),
            Arc::new(Float32Array::from(vec![expected.next_lambda])),
            Arc::new(Float32Array::from(vec![expected.next_min_score])),
            Arc::new(UInt32Array::from(vec![expected.next_max_context_chars])),
            Arc::new(StringArray::from(vec![expected.tuning_reason.as_str()])),
            Arc::new(Float32Array::from(vec![expected.confidence])),
            Arc::new(StringArray::from(vec![expected.schema_version.as_str()])),
        ],
    )
    .unwrap_or_else(|error| panic!("response batch should build: {error}"));

    let rows = decode_memory_julia_plan_tuning_advice_rows(&[batch])
        .unwrap_or_else(|error| panic!("response rows should decode: {error}"));
    assert_eq!(rows, vec![expected]);
}

#[test]
fn validate_memory_julia_plan_tuning_response_batch_rejects_invalid_confidence() {
    let batch = RecordBatch::try_new(
        memory_julia_plan_tuning_response_schema(),
        vec![
            Arc::new(StringArray::from(vec!["repo"])),
            Arc::new(UInt32Array::from(vec![8])),
            Arc::new(UInt32Array::from(vec![4])),
            Arc::new(Float32Array::from(vec![0.65])),
            Arc::new(Float32Array::from(vec![0.12])),
            Arc::new(UInt32Array::from(vec![880])),
            Arc::new(StringArray::from(vec!["tightened budget"])),
            Arc::new(Float32Array::from(vec![1.4])),
            Arc::new(StringArray::from(vec!["v1"])),
        ],
    )
    .unwrap_or_else(|error| panic!("response batch should build: {error}"));

    let Err(error) = validate_memory_julia_plan_tuning_response_batch(&batch) else {
        panic!("confidence above one should fail");
    };
    assert!(error.contains("finite values in [0, 1]"));
}

#[test]
fn validate_memory_julia_plan_tuning_response_batch_rejects_invalid_shape() {
    let batch = RecordBatch::try_new(
        memory_julia_plan_tuning_response_schema(),
        vec![
            Arc::new(StringArray::from(vec!["repo"])),
            Arc::new(UInt32Array::from(vec![4])),
            Arc::new(UInt32Array::from(vec![5])),
            Arc::new(Float32Array::from(vec![0.65])),
            Arc::new(Float32Array::from(vec![0.12])),
            Arc::new(UInt32Array::from(vec![880])),
            Arc::new(StringArray::from(vec!["bad tuning"])),
            Arc::new(Float32Array::from(vec![0.7])),
            Arc::new(StringArray::from(vec!["v1"])),
        ],
    )
    .unwrap_or_else(|error| panic!("response batch should build: {error}"));

    let Err(error) = validate_memory_julia_plan_tuning_response_batch(&batch) else {
        panic!("next_k2 above next_k1 should fail");
    };
    assert!(error.contains("next_k2 greater than next_k1"));
}
