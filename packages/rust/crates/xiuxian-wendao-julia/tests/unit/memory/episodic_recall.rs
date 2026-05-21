use super::{
    MEMORY_JULIA_EPISODIC_RECALL_REQUEST_COLUMNS, MemoryJuliaEpisodicRecallRequestRow,
    build_memory_julia_episodic_recall_request_batch,
    decode_memory_julia_episodic_recall_score_rows, memory_julia_episodic_recall_response_schema,
    validate_memory_julia_episodic_recall_response_batch,
};
use arrow::array::{Float32Array, StringArray};
use arrow::record_batch::RecordBatch;
use std::sync::Arc;

fn sample_request_row() -> MemoryJuliaEpisodicRecallRequestRow {
    MemoryJuliaEpisodicRecallRequestRow {
        query_id: "query-1".into(),
        scenario_pack: Some("searchinfra".to_string()),
        scope: "repo".to_string(),
        query_text: Some("find workaround".to_string()),
        query_embedding: vec![0.2, 0.4, 0.6],
        candidate_id: "episode-1".into(),
        intent_embedding: vec![0.1, 0.3, 0.5],
        q_value: 0.75,
        success_count: 4,
        failure_count: 1,
        retrieval_count: 5,
        created_at_ms: 1_000.into(),
        updated_at_ms: 2_000.into(),
        k1: 1.0,
        k2: 0.5,
        lambda: 0.7,
        min_score: 0.2,
    }
}

#[test]
fn build_memory_julia_episodic_recall_request_batch_accepts_valid_rows() {
    let batch = build_memory_julia_episodic_recall_request_batch(&[sample_request_row()])
        .unwrap_or_else(|error| panic!("request batch should build: {error}"));
    assert_eq!(batch.num_rows(), 1);
    assert_eq!(
        batch
            .schema()
            .fields()
            .iter()
            .map(|field| field.name().clone())
            .collect::<Vec<_>>(),
        MEMORY_JULIA_EPISODIC_RECALL_REQUEST_COLUMNS
            .iter()
            .map(std::string::ToString::to_string)
            .collect::<Vec<_>>()
    );
    assert!(batch.column_by_name("query_embedding").is_some());
}

#[test]
fn build_memory_julia_episodic_recall_request_batch_rejects_invalid_rows() {
    let mut row = sample_request_row();
    row.intent_embedding.clear();
    let Err(error) = build_memory_julia_episodic_recall_request_batch(&[row]) else {
        panic!("empty candidate embedding should fail");
    };
    assert!(error.to_string().contains("empty embedding"));
}

#[test]
fn decode_memory_julia_episodic_recall_score_rows_decodes_valid_batches() {
    let batch = RecordBatch::try_new(
        memory_julia_episodic_recall_response_schema(),
        vec![
            Arc::new(StringArray::from(vec!["query-1"])),
            Arc::new(StringArray::from(vec!["episode-1"])),
            Arc::new(Float32Array::from(vec![0.9])),
            Arc::new(Float32Array::from(vec![0.7])),
            Arc::new(Float32Array::from(vec![0.8])),
            Arc::new(Float32Array::from(vec![0.95])),
            Arc::new(StringArray::from(vec![Some("semantic+utility")])),
            Arc::new(StringArray::from(vec![Some("shadow")])),
            Arc::new(StringArray::from(vec!["v1"])),
        ],
    )
    .unwrap_or_else(|error| panic!("response batch should build: {error}"));

    let rows = decode_memory_julia_episodic_recall_score_rows(&[batch])
        .unwrap_or_else(|error| panic!("response rows should decode: {error}"));
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].query_id, "query-1");
    assert_eq!(rows[0].candidate_id, "episode-1");
    assert_eq!(rows[0].ranking_reason.as_deref(), Some("semantic+utility"));
    assert_eq!(rows[0].retrieval_mode.as_deref(), Some("shadow"));
    assert_eq!(rows[0].schema_version, "v1");
}

#[test]
fn validate_memory_julia_episodic_recall_response_batch_rejects_invalid_confidence() {
    let batch = RecordBatch::try_new(
        memory_julia_episodic_recall_response_schema(),
        vec![
            Arc::new(StringArray::from(vec!["query-1"])),
            Arc::new(StringArray::from(vec!["episode-1"])),
            Arc::new(Float32Array::from(vec![0.9])),
            Arc::new(Float32Array::from(vec![0.7])),
            Arc::new(Float32Array::from(vec![0.8])),
            Arc::new(Float32Array::from(vec![1.5])),
            Arc::new(StringArray::from(vec![None::<&str>])),
            Arc::new(StringArray::from(vec![None::<&str>])),
            Arc::new(StringArray::from(vec!["v1"])),
        ],
    )
    .unwrap_or_else(|error| panic!("response batch should build: {error}"));

    let Err(error) = validate_memory_julia_episodic_recall_response_batch(&batch) else {
        panic!("confidence above one should fail");
    };
    assert!(error.contains("confidence outside [0, 1]"));
}
