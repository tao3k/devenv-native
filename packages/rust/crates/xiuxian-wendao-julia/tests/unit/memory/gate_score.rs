use super::{
    MEMORY_JULIA_GATE_SCORE_REQUEST_COLUMNS, MemoryJuliaGateScoreRecommendationRow,
    MemoryJuliaGateScoreRequestRow, build_memory_julia_gate_score_request_batch,
    decode_memory_julia_gate_score_recommendation_rows, memory_julia_gate_score_response_schema,
    validate_memory_julia_gate_score_response_batch,
};
use arrow::array::{Float32Array, StringArray};
use arrow::record_batch::RecordBatch;
use std::sync::Arc;

fn sample_request_row() -> MemoryJuliaGateScoreRequestRow {
    MemoryJuliaGateScoreRequestRow {
        memory_id: "episode-1".to_string(),
        scenario_pack: Some("searchinfra".to_string()),
        react_revalidation_score: 0.91,
        graph_consistency_score: 0.88,
        omega_alignment_score: 0.93,
        q_value: 0.84,
        usage_count: 5,
        failure_rate: 0.10,
        ttl_score: 0.72,
        current_state: "active".into(),
    }
}

fn sample_recommendation_row() -> MemoryJuliaGateScoreRecommendationRow {
    MemoryJuliaGateScoreRecommendationRow {
        memory_id: "episode-1".to_string(),
        verdict: "promote_to_working_knowledge".to_string(),
        confidence: 0.93,
        utility_score: 0.87,
        ttl_score: 0.72,
        next_action: "promote_to_working_knowledge".to_string(),
        reason: "utility and ttl exceed working-knowledge promotion threshold".to_string(),
        schema_version: "v1".to_string(),
    }
}

#[test]
fn build_memory_julia_gate_score_request_batch_accepts_valid_rows() {
    let batch = build_memory_julia_gate_score_request_batch(&[sample_request_row()])
        .unwrap_or_else(|error| panic!("request batch should build: {error}"));
    assert_eq!(batch.num_rows(), 1);
    assert_eq!(
        batch
            .schema()
            .fields()
            .iter()
            .map(|field| field.name().clone())
            .collect::<Vec<_>>(),
        MEMORY_JULIA_GATE_SCORE_REQUEST_COLUMNS
            .iter()
            .map(std::string::ToString::to_string)
            .collect::<Vec<_>>()
    );
    assert!(batch.column_by_name("current_state").is_some());
}

#[test]
fn build_memory_julia_gate_score_request_batch_rejects_invalid_state() {
    let mut row = sample_request_row();
    row.current_state = "unknown".into();
    let Err(error) = build_memory_julia_gate_score_request_batch(&[row]) else {
        panic!("unsupported lifecycle state should fail");
    };
    assert!(error.to_string().contains("unsupported value"));
}

#[test]
fn decode_memory_julia_gate_score_recommendation_rows_decodes_valid_batches() {
    let expected = sample_recommendation_row();
    let batch = RecordBatch::try_new(
        memory_julia_gate_score_response_schema(),
        vec![
            Arc::new(StringArray::from(vec![expected.memory_id.as_str()])),
            Arc::new(StringArray::from(vec![expected.verdict.as_str()])),
            Arc::new(Float32Array::from(vec![expected.confidence])),
            Arc::new(Float32Array::from(vec![expected.utility_score])),
            Arc::new(Float32Array::from(vec![expected.ttl_score])),
            Arc::new(StringArray::from(vec![expected.next_action.as_str()])),
            Arc::new(StringArray::from(vec![expected.reason.as_str()])),
            Arc::new(StringArray::from(vec![expected.schema_version.as_str()])),
        ],
    )
    .unwrap_or_else(|error| panic!("response batch should build: {error}"));

    let rows = decode_memory_julia_gate_score_recommendation_rows(&[batch])
        .unwrap_or_else(|error| panic!("response rows should decode: {error}"));
    assert_eq!(rows, vec![expected]);
}

#[test]
fn validate_memory_julia_gate_score_response_batch_rejects_invalid_confidence() {
    let batch = RecordBatch::try_new(
        memory_julia_gate_score_response_schema(),
        vec![
            Arc::new(StringArray::from(vec!["episode-1"])),
            Arc::new(StringArray::from(vec!["retain"])),
            Arc::new(Float32Array::from(vec![1.2])),
            Arc::new(Float32Array::from(vec![0.64])),
            Arc::new(Float32Array::from(vec![0.58])),
            Arc::new(StringArray::from(vec!["retain"])),
            Arc::new(StringArray::from(vec!["stay active"])),
            Arc::new(StringArray::from(vec!["v1"])),
        ],
    )
    .unwrap_or_else(|error| panic!("response batch should build: {error}"));

    let Err(error) = validate_memory_julia_gate_score_response_batch(&batch) else {
        panic!("confidence above one should fail");
    };
    assert!(error.contains("finite values in [0, 1]"));
}

#[test]
fn validate_memory_julia_gate_score_response_batch_rejects_unknown_verdict() {
    let batch = RecordBatch::try_new(
        memory_julia_gate_score_response_schema(),
        vec![
            Arc::new(StringArray::from(vec!["episode-1"])),
            Arc::new(StringArray::from(vec!["promote"])),
            Arc::new(Float32Array::from(vec![0.82])),
            Arc::new(Float32Array::from(vec![0.79])),
            Arc::new(Float32Array::from(vec![0.61])),
            Arc::new(StringArray::from(vec!["promote_to_working_knowledge"])),
            Arc::new(StringArray::from(vec!["promote candidate"])),
            Arc::new(StringArray::from(vec!["v1"])),
        ],
    )
    .unwrap_or_else(|error| panic!("response batch should build: {error}"));

    let Err(error) = validate_memory_julia_gate_score_response_batch(&batch) else {
        panic!("unsupported verdict should fail");
    };
    assert!(error.contains("unsupported value"));
}
