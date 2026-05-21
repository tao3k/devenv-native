use super::{
    validate_memory_julia_compute_request_batches, validate_memory_julia_compute_response_batches,
};
use crate::memory::{
    MemoryJuliaComputeProfile, MemoryJuliaGateScoreRequestRow,
    build_memory_julia_gate_score_request_batch, memory_julia_gate_score_response_schema,
};
use arrow::array::{Float32Array, StringArray};
use arrow::record_batch::RecordBatch;
use std::sync::Arc;

#[test]
fn validate_memory_julia_compute_request_batches_dispatches_by_profile() {
    let batch = build_memory_julia_gate_score_request_batch(&[MemoryJuliaGateScoreRequestRow {
        memory_id: "memory-1".to_string(),
        scenario_pack: Some("searchinfra".to_string()),
        react_revalidation_score: 0.9,
        graph_consistency_score: 0.8,
        omega_alignment_score: 0.85,
        q_value: 0.75,
        usage_count: 4,
        failure_rate: 0.25,
        ttl_score: 0.7,
        current_state: "active".into(),
    }])
    .unwrap_or_else(|error| panic!("request batch should build: {error}"));

    validate_memory_julia_compute_request_batches(
        MemoryJuliaComputeProfile::MemoryGateScore,
        &[batch],
    )
    .unwrap_or_else(|error| panic!("request validation should pass: {error}"));
}

#[test]
fn validate_memory_julia_compute_response_batches_dispatches_by_profile() {
    let batch = RecordBatch::try_new(
        memory_julia_gate_score_response_schema(),
        vec![
            Arc::new(StringArray::from(vec!["memory-1"])),
            Arc::new(StringArray::from(vec!["retain"])),
            Arc::new(Float32Array::from(vec![0.9_f32])),
            Arc::new(Float32Array::from(vec![0.75_f32])),
            Arc::new(Float32Array::from(vec![0.7_f32])),
            Arc::new(StringArray::from(vec!["keep"])),
            Arc::new(StringArray::from(vec!["stable"])),
            Arc::new(StringArray::from(vec!["v1"])),
        ],
    )
    .unwrap_or_else(|error| panic!("response batch should build: {error}"));

    validate_memory_julia_compute_response_batches(
        MemoryJuliaComputeProfile::MemoryGateScore,
        &[batch],
    )
    .unwrap_or_else(|error| panic!("response validation should pass: {error}"));
}
