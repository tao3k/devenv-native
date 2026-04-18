use std::sync::Arc;

use arrow_schema::{DataType, Field, Schema};

use super::super::{
    RERANK_REQUEST_DOC_ID_COLUMN, RERANK_REQUEST_EMBEDDING_COLUMN,
    RERANK_REQUEST_QUERY_EMBEDDING_COLUMN, RERANK_REQUEST_VECTOR_SCORE_COLUMN,
    validate_rerank_request_batch, validate_rerank_request_schema,
};
use super::support::build_rerank_request_batch;

#[test]
fn rerank_request_schema_validation_accepts_stable_shape() {
    let schema = Schema::new(vec![
        Field::new(RERANK_REQUEST_DOC_ID_COLUMN, DataType::Utf8, false),
        Field::new(RERANK_REQUEST_VECTOR_SCORE_COLUMN, DataType::Float32, false),
        Field::new(
            RERANK_REQUEST_EMBEDDING_COLUMN,
            DataType::FixedSizeList(Arc::new(Field::new("item", DataType::Float32, true)), 3),
            false,
        ),
        Field::new(
            RERANK_REQUEST_QUERY_EMBEDDING_COLUMN,
            DataType::FixedSizeList(Arc::new(Field::new("item", DataType::Float32, true)), 3),
            false,
        ),
    ]);

    assert!(validate_rerank_request_schema(&schema, 3).is_ok());
}

#[test]
fn rerank_request_schema_validation_rejects_wrong_scalar_type() {
    let schema = Schema::new(vec![
        Field::new(RERANK_REQUEST_DOC_ID_COLUMN, DataType::Utf8, false),
        Field::new(RERANK_REQUEST_VECTOR_SCORE_COLUMN, DataType::Float64, false),
        Field::new(
            RERANK_REQUEST_EMBEDDING_COLUMN,
            DataType::FixedSizeList(Arc::new(Field::new("item", DataType::Float32, true)), 3),
            false,
        ),
        Field::new(
            RERANK_REQUEST_QUERY_EMBEDDING_COLUMN,
            DataType::FixedSizeList(Arc::new(Field::new("item", DataType::Float32, true)), 3),
            false,
        ),
    ]);

    assert_eq!(
        validate_rerank_request_schema(&schema, 3),
        Err("rerank request column `vector_score` must be Float32".to_string())
    );
}

#[test]
fn rerank_request_schema_validation_rejects_dimension_drift() {
    let schema = Schema::new(vec![
        Field::new(RERANK_REQUEST_DOC_ID_COLUMN, DataType::Utf8, false),
        Field::new(RERANK_REQUEST_VECTOR_SCORE_COLUMN, DataType::Float32, false),
        Field::new(
            RERANK_REQUEST_EMBEDDING_COLUMN,
            DataType::FixedSizeList(Arc::new(Field::new("item", DataType::Float32, true)), 2),
            false,
        ),
        Field::new(
            RERANK_REQUEST_QUERY_EMBEDDING_COLUMN,
            DataType::FixedSizeList(Arc::new(Field::new("item", DataType::Float32, true)), 2),
            false,
        ),
    ]);

    assert_eq!(
        validate_rerank_request_schema(&schema, 3),
        Err("rerank request column `embedding` must use dimension 3, got 2".to_string())
    );
}

#[test]
fn rerank_request_batch_validation_accepts_stable_semantics() {
    let batch = build_rerank_request_batch(
        vec!["doc-1", "doc-2"],
        vec![0.9_f32, 0.8_f32],
        vec![
            vec![0.1_f32, 0.2_f32, 0.3_f32],
            vec![0.4_f32, 0.5_f32, 0.6_f32],
        ],
        vec![
            vec![0.7_f32, 0.8_f32, 0.9_f32],
            vec![0.7_f32, 0.8_f32, 0.9_f32],
        ],
    );

    assert!(validate_rerank_request_batch(&batch, 3).is_ok());
}

#[test]
fn rerank_request_batch_validation_rejects_blank_doc_id() {
    let batch = build_rerank_request_batch(
        vec![" "],
        vec![0.9_f32],
        vec![vec![0.1_f32, 0.2_f32, 0.3_f32]],
        vec![vec![0.7_f32, 0.8_f32, 0.9_f32]],
    );

    assert_eq!(
        validate_rerank_request_batch(&batch, 3),
        Err(
            "rerank request column `doc_id` must not contain blank values; row 0 is blank"
                .to_string()
        )
    );
}

#[test]
fn rerank_request_batch_validation_rejects_duplicate_doc_id() {
    let batch = build_rerank_request_batch(
        vec!["doc-1", "doc-1"],
        vec![0.9_f32, 0.8_f32],
        vec![
            vec![0.1_f32, 0.2_f32, 0.3_f32],
            vec![0.4_f32, 0.5_f32, 0.6_f32],
        ],
        vec![
            vec![0.7_f32, 0.8_f32, 0.9_f32],
            vec![0.7_f32, 0.8_f32, 0.9_f32],
        ],
    );

    assert_eq!(
        validate_rerank_request_batch(&batch, 3),
        Err(
            "rerank request column `doc_id` must be unique across one batch; row 1 duplicates `doc-1`"
                .to_string()
        )
    );
}

#[test]
fn rerank_request_batch_validation_rejects_out_of_range_vector_score() {
    let batch = build_rerank_request_batch(
        vec!["doc-1"],
        vec![1.2_f32],
        vec![vec![0.1_f32, 0.2_f32, 0.3_f32]],
        vec![vec![0.7_f32, 0.8_f32, 0.9_f32]],
    );

    assert_eq!(
        validate_rerank_request_batch(&batch, 3),
        Err(
            "rerank request column `vector_score` must stay within inclusive range [0.0, 1.0]; row 0 is 1.2"
                .to_string()
        )
    );
}

#[test]
fn rerank_request_batch_validation_rejects_query_embedding_drift() {
    let batch = build_rerank_request_batch(
        vec!["doc-1", "doc-2"],
        vec![0.9_f32, 0.8_f32],
        vec![
            vec![0.1_f32, 0.2_f32, 0.3_f32],
            vec![0.4_f32, 0.5_f32, 0.6_f32],
        ],
        vec![
            vec![0.7_f32, 0.8_f32, 0.9_f32],
            vec![1.0_f32, 1.1_f32, 1.2_f32],
        ],
    );

    assert_eq!(
        validate_rerank_request_batch(&batch, 3),
        Err(
            "rerank request column `query_embedding` must remain stable across all rows; row 1 differs from row 0"
                .to_string()
        )
    );
}
