use std::sync::Arc;

use arrow_array::types::Float32Type;
use arrow_array::{
    FixedSizeListArray, Float32Array, Float64Array, Int32Array, RecordBatch, StringArray,
};
use arrow_schema::{DataType, Field, Schema};

use super::super::{
    RERANK_REQUEST_DOC_ID_COLUMN, RERANK_REQUEST_EMBEDDING_COLUMN,
    RERANK_REQUEST_QUERY_EMBEDDING_COLUMN, RERANK_REQUEST_VECTOR_SCORE_COLUMN,
    RERANK_RESPONSE_DOC_ID_COLUMN, RERANK_RESPONSE_FINAL_SCORE_COLUMN, RERANK_RESPONSE_RANK_COLUMN,
    RERANK_RESPONSE_SEMANTIC_SCORE_COLUMN, RERANK_RESPONSE_VECTOR_SCORE_COLUMN, must_ok,
};

pub(super) fn build_rerank_request_batch(
    doc_ids: Vec<&str>,
    vector_scores: Vec<f32>,
    embeddings: Vec<Vec<f32>>,
    query_embeddings: Vec<Vec<f32>>,
) -> RecordBatch {
    let schema = Arc::new(Schema::new(vec![
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
    ]));

    let embedding_values = embeddings
        .into_iter()
        .map(|row| Some(row.into_iter().map(Some).collect::<Vec<Option<f32>>>()));
    let query_embedding_values = query_embeddings
        .into_iter()
        .map(|row| Some(row.into_iter().map(Some).collect::<Vec<Option<f32>>>()));

    must_ok(
        RecordBatch::try_new(
            schema,
            vec![
                Arc::new(StringArray::from(doc_ids)),
                Arc::new(Float32Array::from(vector_scores)),
                Arc::new(
                    FixedSizeListArray::from_iter_primitive::<Float32Type, _, _>(
                        embedding_values,
                        3,
                    ),
                ),
                Arc::new(
                    FixedSizeListArray::from_iter_primitive::<Float32Type, _, _>(
                        query_embedding_values,
                        3,
                    ),
                ),
            ],
        ),
        "record batch should build",
    )
}

pub(super) fn build_rerank_response_batch(ranks: Vec<i32>, final_scores: Vec<f64>) -> RecordBatch {
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

    must_ok(
        RecordBatch::try_new(
            schema,
            vec![
                Arc::new(StringArray::from(vec!["doc-1", "doc-2"])),
                Arc::new(Float64Array::from(vec![0.91_f64, 0.82_f64])),
                Arc::new(Float64Array::from(vec![0.97_f64, 0.91_f64])),
                Arc::new(Float64Array::from(final_scores)),
                Arc::new(Int32Array::from(ranks)),
            ],
        ),
        "record batch should build",
    )
}
