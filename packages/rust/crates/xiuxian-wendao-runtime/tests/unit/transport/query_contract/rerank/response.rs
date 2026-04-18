use std::sync::Arc;

use arrow_array::{Float64Array, Int32Array, RecordBatch, StringArray};
use arrow_schema::{DataType, Field, Schema};

use super::super::{
    RERANK_RESPONSE_DOC_ID_COLUMN, RERANK_RESPONSE_FINAL_SCORE_COLUMN, RERANK_RESPONSE_RANK_COLUMN,
    RERANK_RESPONSE_SEMANTIC_SCORE_COLUMN, RERANK_RESPONSE_VECTOR_SCORE_COLUMN, must_ok,
    validate_rerank_response_batch, validate_rerank_response_schema,
};
use super::support::build_rerank_response_batch;

#[test]
fn rerank_response_schema_validation_accepts_stable_shape() {
    let schema = Schema::new(vec![
        Field::new(RERANK_RESPONSE_DOC_ID_COLUMN, DataType::Utf8, false),
        Field::new(
            RERANK_RESPONSE_VECTOR_SCORE_COLUMN,
            DataType::Float64,
            false,
        ),
        Field::new(
            RERANK_RESPONSE_SEMANTIC_SCORE_COLUMN,
            DataType::Float64,
            false,
        ),
        Field::new(RERANK_RESPONSE_FINAL_SCORE_COLUMN, DataType::Float64, false),
        Field::new(RERANK_RESPONSE_RANK_COLUMN, DataType::Int32, false),
    ]);

    assert!(validate_rerank_response_schema(&schema).is_ok());
}

#[test]
fn rerank_response_schema_validation_rejects_wrong_rank_type() {
    let schema = Schema::new(vec![
        Field::new(RERANK_RESPONSE_DOC_ID_COLUMN, DataType::Utf8, false),
        Field::new(
            RERANK_RESPONSE_VECTOR_SCORE_COLUMN,
            DataType::Float64,
            false,
        ),
        Field::new(
            RERANK_RESPONSE_SEMANTIC_SCORE_COLUMN,
            DataType::Float64,
            false,
        ),
        Field::new(RERANK_RESPONSE_FINAL_SCORE_COLUMN, DataType::Float64, false),
        Field::new(RERANK_RESPONSE_RANK_COLUMN, DataType::UInt32, false),
    ]);

    assert_eq!(
        validate_rerank_response_schema(&schema),
        Err("rerank response column `rank` must be Int32".to_string())
    );
}

#[test]
fn rerank_response_batch_validation_accepts_stable_semantics() {
    let batch = build_rerank_response_batch(vec![1_i32, 2_i32], vec![0.97_f64, 0.91_f64]);
    assert!(validate_rerank_response_batch(&batch).is_ok());
}

#[test]
fn rerank_response_batch_validation_rejects_duplicate_rank() {
    let batch = build_rerank_response_batch(vec![1_i32, 1_i32], vec![0.97_f64, 0.91_f64]);

    assert_eq!(
        validate_rerank_response_batch(&batch),
        Err(
            "rerank response column `rank` must be unique across one batch; row 1 duplicates `1`"
                .to_string()
        )
    );
}

#[test]
fn rerank_response_batch_validation_rejects_out_of_range_final_score() {
    let schema = Arc::new(Schema::new(vec![
        Field::new(RERANK_RESPONSE_DOC_ID_COLUMN, DataType::Utf8, false),
        Field::new(
            RERANK_RESPONSE_VECTOR_SCORE_COLUMN,
            DataType::Float64,
            false,
        ),
        Field::new(
            RERANK_RESPONSE_SEMANTIC_SCORE_COLUMN,
            DataType::Float64,
            false,
        ),
        Field::new(RERANK_RESPONSE_FINAL_SCORE_COLUMN, DataType::Float64, false),
        Field::new(RERANK_RESPONSE_RANK_COLUMN, DataType::Int32, false),
    ]));
    let batch = must_ok(
        RecordBatch::try_new(
            schema,
            vec![
                Arc::new(StringArray::from(vec!["doc-1"])),
                Arc::new(Float64Array::from(vec![0.9_f64])),
                Arc::new(Float64Array::from(vec![0.95_f64])),
                Arc::new(Float64Array::from(vec![1.2_f64])),
                Arc::new(Int32Array::from(vec![1_i32])),
            ],
        ),
        "record batch should build",
    );

    assert_eq!(
        validate_rerank_response_batch(&batch),
        Err(
            "rerank response column `final_score` must stay within inclusive range [0.0, 1.0]; row 0 is 1.2"
                .to_string()
        )
    );
}
