use std::sync::Arc;

use arrow_array::{RecordBatch, StringArray};
use arrow_schema::{DataType, Field, Schema};

use super::support::{
    err_or_panic, invalid_response_missing_analyzer_score_batch, response_batch_with_duplicates,
    response_batch_without_trace_id,
};
use crate::transport::plugin_arrow_exchange::{
    PluginArrowScoreRow, attach_plugin_arrow_request_metadata, decode_plugin_arrow_score_rows,
    plugin_arrow_request_trace_id, validate_plugin_arrow_response_batches,
};

#[test]
fn plugin_arrow_request_trace_id_normalizes_query_text() {
    assert_eq!(
        plugin_arrow_request_trace_id("xiuxian-wendao-julia", "  alpha   signal "),
        "plugin-rerank:xiuxian-wendao-julia:alpha_signal"
    );
    assert_eq!(
        plugin_arrow_request_trace_id("xiuxian-wendao-julia", ""),
        "plugin-rerank:xiuxian-wendao-julia:query"
    );
}

#[test]
fn attach_plugin_arrow_request_metadata_sets_schema_metadata() {
    let batch = RecordBatch::try_new(
        Arc::new(Schema::new(vec![Field::new(
            "doc_id",
            DataType::Utf8,
            false,
        )])),
        vec![Arc::new(StringArray::from(vec!["doc-1"]))],
    )
    .unwrap_or_else(|error| panic!("batch: {error}"));

    let traced_batch = attach_plugin_arrow_request_metadata(
        &batch,
        plugin_arrow_request_trace_id("xiuxian-wendao-julia", "alpha signal").as_str(),
        "v1",
    )
    .unwrap_or_else(|error| panic!("metadata: {error}"));

    assert_eq!(
        traced_batch.schema().metadata().get("trace_id"),
        Some(&"plugin-rerank:xiuxian-wendao-julia:alpha_signal".to_string())
    );
    assert_eq!(
        traced_batch
            .schema()
            .metadata()
            .get("wendao.schema_version"),
        Some(&"v1".to_string())
    );
}

#[test]
fn decode_plugin_arrow_score_rows_materializes_doc_scores() {
    let rows = decode_plugin_arrow_score_rows(&[response_batch_without_trace_id()])
        .unwrap_or_else(|error| panic!("decode should work: {error}"));

    assert_eq!(rows.len(), 2);
    assert_eq!(
        rows.get("doc-a"),
        Some(&PluginArrowScoreRow {
            doc_id: "doc-a".to_string(),
            analyzer_score: 0.2,
            final_score: 0.5,
            trace_id: None,
        })
    );
    assert_eq!(
        rows.get("doc-b"),
        Some(&PluginArrowScoreRow {
            doc_id: "doc-b".to_string(),
            analyzer_score: 0.7,
            final_score: 0.9,
            trace_id: None,
        })
    );
}

#[test]
fn decode_plugin_arrow_score_rows_rejects_missing_columns() {
    let error = err_or_panic(
        decode_plugin_arrow_score_rows(&[invalid_response_missing_analyzer_score_batch()]),
        "decode should fail",
    );
    assert!(
        error
            .to_string()
            .contains("missing required Float64 column `analyzer_score`"),
        "unexpected error: {error}"
    );
}

#[test]
fn validate_plugin_arrow_response_batches_accepts_v1_shape() {
    let result = validate_plugin_arrow_response_batches(&[response_batch_without_trace_id()]);
    assert!(result.is_ok(), "expected valid plugin response: {result:?}");
}

#[test]
fn validate_plugin_arrow_response_batches_rejects_duplicates_and_missing_columns() {
    let duplicate_error = err_or_panic(
        validate_plugin_arrow_response_batches(&[response_batch_with_duplicates()]),
        "duplicate doc_id must fail",
    );
    assert!(
        duplicate_error
            .to_string()
            .contains("duplicate `doc_id` in plugin analyzer response"),
        "unexpected duplicate error: {duplicate_error}"
    );

    let missing_error = err_or_panic(
        validate_plugin_arrow_response_batches(&[invalid_response_missing_analyzer_score_batch()]),
        "missing analyzer_score must fail",
    );
    assert!(
        missing_error
            .to_string()
            .contains("missing required Float64 column `analyzer_score`"),
        "unexpected missing-column error: {missing_error}"
    );
}
