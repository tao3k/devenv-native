use std::sync::Arc;

use xiuxian_db_store::{
    LanceFloat64Array as Float64Array, LanceInt32Array as Int32Array,
    LanceStringArray as StringArray,
};

use crate::transport::{
    RerankFlightRouteHandler, RerankScoreWeights, validate_rerank_top_k_header,
};

use super::assertions::{lance_batch_column, metadata_value, must_err, must_ok};

fn build_rerank_request_batch() -> arrow_array::RecordBatch {
    use arrow_array::types::Float32Type;
    use arrow_array::{FixedSizeListArray, Float32Array, RecordBatch, StringArray};
    use arrow_schema::{DataType, Field, Schema};

    must_ok(
        RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new("doc_id", DataType::Utf8, false),
                Field::new("vector_score", DataType::Float32, false),
                Field::new(
                    "embedding",
                    DataType::FixedSizeList(
                        Arc::new(Field::new("item", DataType::Float32, true)),
                        3,
                    ),
                    false,
                ),
                Field::new(
                    "query_embedding",
                    DataType::FixedSizeList(
                        Arc::new(Field::new("item", DataType::Float32, true)),
                        3,
                    ),
                    false,
                ),
            ])),
            vec![
                Arc::new(StringArray::from(vec!["doc-0", "doc-1"])),
                Arc::new(Float32Array::from(vec![0.5_f32, 0.8_f32])),
                Arc::new(
                    FixedSizeListArray::from_iter_primitive::<Float32Type, _, _>(
                        vec![
                            Some(vec![Some(1.0_f32), Some(0.0_f32), Some(0.0_f32)]),
                            Some(vec![Some(0.0_f32), Some(1.0_f32), Some(0.0_f32)]),
                        ],
                        3,
                    ),
                ),
                Arc::new(
                    FixedSizeListArray::from_iter_primitive::<Float32Type, _, _>(
                        vec![
                            Some(vec![Some(1.0_f32), Some(0.0_f32), Some(0.0_f32)]),
                            Some(vec![Some(1.0_f32), Some(0.0_f32), Some(0.0_f32)]),
                        ],
                        3,
                    ),
                ),
            ],
        ),
        "request batch should build",
    )
}

#[test]
fn rerank_route_handler_scores_and_ranks_semantic_candidates() {
    let request_batch = build_rerank_request_batch();
    let handler = must_ok(RerankFlightRouteHandler::new(3), "handler should build");

    let response = must_ok(
        handler.handle_exchange_batches(std::slice::from_ref(&request_batch), None, None),
        "rerank route handler should score request batches",
    );

    let doc_ids =
        lance_batch_column::<StringArray>(&response, "doc_id", "doc_id should decode as Utf8");
    let vector_scores = lance_batch_column::<Float64Array>(
        &response,
        "vector_score",
        "vector_score should decode as Float64",
    );
    let semantic_scores = lance_batch_column::<Float64Array>(
        &response,
        "semantic_score",
        "semantic_score should decode as Float64",
    );
    let final_scores = lance_batch_column::<Float64Array>(
        &response,
        "final_score",
        "final_score should decode as Float64",
    );
    let ranks = lance_batch_column::<Int32Array>(&response, "rank", "rank should decode as Int32");

    assert_eq!(doc_ids.value(0), "doc-0");
    assert_eq!(doc_ids.value(1), "doc-1");
    assert!((vector_scores.value(0) - 0.5).abs() < 1e-6);
    assert!((vector_scores.value(1) - 0.8).abs() < 1e-6);
    assert!((semantic_scores.value(0) - 1.0).abs() < 1e-6);
    assert!((semantic_scores.value(1) - 0.5).abs() < 1e-6);
    assert!((final_scores.value(0) - 0.8).abs() < 1e-6);
    assert!((final_scores.value(1) - 0.62).abs() < 1e-6);
    assert_eq!(ranks.value(0), 1);
    assert_eq!(ranks.value(1), 2);
}

#[test]
fn rerank_route_handler_respects_runtime_weight_policy() {
    let request_batch = build_rerank_request_batch();
    let handler = must_ok(
        RerankFlightRouteHandler::new_with_weights(
            3,
            must_ok(RerankScoreWeights::new(0.9, 0.1), "weights should validate"),
        ),
        "handler should build",
    );

    let response = must_ok(
        handler.handle_exchange_batches(std::slice::from_ref(&request_batch), None, None),
        "rerank route handler should score request batches",
    );

    let doc_ids =
        lance_batch_column::<StringArray>(&response, "doc_id", "doc_id should decode as Utf8");
    let final_scores = lance_batch_column::<Float64Array>(
        &response,
        "final_score",
        "final_score should decode as Float64",
    );

    assert_eq!(doc_ids.value(0), "doc-1");
    assert_eq!(doc_ids.value(1), "doc-0");
    assert!((final_scores.value(0) - 0.77).abs() < 1e-6);
    assert!((final_scores.value(1) - 0.55).abs() < 1e-6);
}

#[test]
fn rerank_route_handler_rejects_zero_dimension() {
    let error = must_err(
        RerankFlightRouteHandler::new(0),
        "zero-dimension handler construction should fail",
    );
    assert_eq!(
        error,
        "rerank route expected_dimension must be greater than zero"
    );
}

#[test]
fn rerank_route_handler_applies_top_k_after_scoring() {
    let request_batch = build_rerank_request_batch();
    let handler = must_ok(RerankFlightRouteHandler::new(3), "handler should build");

    let response = must_ok(
        handler.handle_exchange_batches(std::slice::from_ref(&request_batch), Some(1), None),
        "rerank route handler should truncate scored request batches",
    );

    let doc_ids =
        lance_batch_column::<StringArray>(&response, "doc_id", "doc_id should decode as Utf8");
    let ranks = lance_batch_column::<Int32Array>(&response, "rank", "rank should decode as Int32");

    assert_eq!(response.num_rows(), 1);
    assert_eq!(doc_ids.value(0), "doc-0");
    assert_eq!(ranks.value(0), 1);
}

#[test]
fn rerank_route_handler_preserves_full_result_when_top_k_exceeds_candidate_count() {
    let request_batch = build_rerank_request_batch();
    let handler = must_ok(RerankFlightRouteHandler::new(3), "handler should build");

    let response = must_ok(
        handler.handle_exchange_batches(std::slice::from_ref(&request_batch), Some(10), None),
        "rerank route handler should preserve all scored request batches",
    );

    let doc_ids =
        lance_batch_column::<StringArray>(&response, "doc_id", "doc_id should decode as Utf8");
    let ranks = lance_batch_column::<Int32Array>(&response, "rank", "rank should decode as Int32");

    assert_eq!(response.num_rows(), 2);
    assert_eq!(doc_ids.value(0), "doc-0");
    assert_eq!(doc_ids.value(1), "doc-1");
    assert_eq!(ranks.value(0), 1);
    assert_eq!(ranks.value(1), 2);
}

#[test]
fn rerank_route_handler_preserves_full_result_when_top_k_matches_candidate_count() {
    let request_batch = build_rerank_request_batch();
    let handler = must_ok(RerankFlightRouteHandler::new(3), "handler should build");

    let response = must_ok(
        handler.handle_exchange_batches(std::slice::from_ref(&request_batch), Some(2), None),
        "rerank route handler should preserve all scored request batches",
    );

    let doc_ids =
        lance_batch_column::<StringArray>(&response, "doc_id", "doc_id should decode as Utf8");
    let ranks = lance_batch_column::<Int32Array>(&response, "rank", "rank should decode as Int32");

    assert_eq!(response.num_rows(), 2);
    assert_eq!(doc_ids.value(0), "doc-0");
    assert_eq!(doc_ids.value(1), "doc-1");
    assert_eq!(ranks.value(0), 1);
    assert_eq!(ranks.value(1), 2);
}

#[test]
fn validate_rerank_top_k_header_rejects_zero() {
    let mut metadata = tonic::metadata::MetadataMap::new();
    metadata.insert(
        crate::transport::WENDAO_RERANK_TOP_K_HEADER,
        metadata_value("0", "metadata should parse"),
    );

    let error = must_err(
        validate_rerank_top_k_header(&metadata),
        "zero rerank top_k should fail",
    );

    assert_eq!(
        error.message(),
        "rerank top_k header `x-wendao-rerank-top-k` must be greater than zero"
    );
}

#[test]
fn validate_rerank_top_k_header_rejects_non_numeric_values() {
    let mut metadata = tonic::metadata::MetadataMap::new();
    metadata.insert(
        crate::transport::WENDAO_RERANK_TOP_K_HEADER,
        metadata_value("abc", "metadata should parse"),
    );

    let error = must_err(
        validate_rerank_top_k_header(&metadata),
        "non-numeric rerank top_k should fail",
    );

    assert_eq!(
        error.message(),
        "invalid rerank top_k header `x-wendao-rerank-top-k`: abc"
    );
}

#[test]
fn validate_rerank_top_k_header_rejects_blank_values() {
    let mut metadata = tonic::metadata::MetadataMap::new();
    metadata.insert(
        crate::transport::WENDAO_RERANK_TOP_K_HEADER,
        metadata_value("", "metadata should parse"),
    );

    let error = must_err(
        validate_rerank_top_k_header(&metadata),
        "blank rerank top_k should fail",
    );

    assert_eq!(
        error.message(),
        "invalid rerank top_k header `x-wendao-rerank-top-k`: "
    );
}
