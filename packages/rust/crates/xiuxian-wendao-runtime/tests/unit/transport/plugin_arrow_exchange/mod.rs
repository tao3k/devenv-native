use std::{collections::BTreeMap, sync::Arc};

use crate::transport::RERANK_ROUTE;
use arrow_array::{Float64Array, RecordBatch, StringArray};
use arrow_schema::{DataType, Field, Schema};
use xiuxian_vector::VectorStore;
use xiuxian_wendao_core::repo_intelligence::{
    julia_arrow_request_schema, julia_arrow_response_schema,
};
use xiuxian_wendao_core::{
    capabilities::{ContractVersion, PluginCapabilityBinding, PluginProviderSelector},
    ids::{CapabilityId, PluginId},
    transport::{PluginTransportEndpoint, PluginTransportKind},
};

use super::attach_plugin_arrow_request_metadata;
use super::{
    PluginArrowRequestBatchBuildError, PluginArrowRequestRow, PluginArrowScoreRow,
    PluginArrowScoredCandidate, PluginArrowVectorStoreRequestBuildError,
    build_plugin_arrow_request_batch, build_plugin_arrow_request_batch_from_embeddings,
    build_plugin_arrow_request_batch_from_embeddings_with_metadata,
    build_plugin_arrow_request_batch_from_vector_store_with_metadata,
    decode_plugin_arrow_score_rows, plugin_arrow_request_trace_id,
    prepare_plugin_arrow_request_rows_from_vector_store, project_plugin_arrow_scored_candidates,
    roundtrip_plugin_arrow_score_rows_with_binding, validate_plugin_arrow_response_batches,
};

fn response_batch_without_trace_id() -> RecordBatch {
    RecordBatch::try_new(
        julia_arrow_response_schema(false),
        vec![
            Arc::new(StringArray::from(vec!["doc-a", "doc-b"])),
            Arc::new(Float64Array::from(vec![0.2, 0.7])),
            Arc::new(Float64Array::from(vec![0.5, 0.9])),
        ],
    )
    .unwrap_or_else(|error| panic!("response batch should build: {error}"))
}

fn sample_binding(base_url: Option<&str>) -> PluginCapabilityBinding {
    PluginCapabilityBinding {
        selector: PluginProviderSelector {
            capability_id: CapabilityId("rerank".to_string()),
            provider: PluginId("xiuxian-wendao-julia".to_string()),
        },
        endpoint: PluginTransportEndpoint {
            base_url: base_url.map(ToString::to_string),
            route: Some(RERANK_ROUTE.to_string()),
            health_route: Some("/healthz".to_string()),
            timeout_secs: Some(5),
            max_in_flight_requests: None,
        },
        launch: None,
        transport: PluginTransportKind::ArrowFlight,
        contract_version: ContractVersion("v2".to_string()),
    }
}

fn response_batch_with_duplicates() -> RecordBatch {
    RecordBatch::try_new(
        julia_arrow_response_schema(false),
        vec![
            Arc::new(StringArray::from(vec!["doc-a", "doc-a"])),
            Arc::new(Float64Array::from(vec![0.2, 0.7])),
            Arc::new(Float64Array::from(vec![0.5, 0.9])),
        ],
    )
    .unwrap_or_else(|error| panic!("duplicate response batch should build: {error}"))
}

fn invalid_response_missing_analyzer_score_batch() -> RecordBatch {
    RecordBatch::try_new(
        Arc::new(Schema::new(vec![
            Field::new("doc_id", DataType::Utf8, false),
            Field::new("final_score", DataType::Float64, false),
        ])),
        vec![
            Arc::new(StringArray::from(vec!["doc-a"])),
            Arc::new(Float64Array::from(vec![0.5])),
        ],
    )
    .unwrap_or_else(|error| panic!("invalid response batch should build: {error}"))
}

fn tempdir_or_panic() -> tempfile::TempDir {
    tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"))
}

fn err_or_panic<T, E>(result: Result<T, E>, failure_message: &str) -> E {
    match result {
        Ok(_) => panic!("{failure_message}"),
        Err(error) => error,
    }
}

#[test]
fn build_plugin_arrow_request_batch_uses_contract_columns() {
    let batch = build_plugin_arrow_request_batch(
        &[
            PluginArrowRequestRow {
                doc_id: "doc-1".to_string(),
                vector_score: 0.3,
                embedding: vec![1.0, 2.0, 3.0],
            },
            PluginArrowRequestRow {
                doc_id: "doc-2".to_string(),
                vector_score: 0.4,
                embedding: vec![4.0, 5.0, 6.0],
            },
        ],
        &[9.0, 8.0, 7.0],
    )
    .unwrap_or_else(|error| panic!("request batch should build: {error}"));

    assert_eq!(batch.num_rows(), 2);
    assert_eq!(batch.schema().field(0).name(), "doc_id");
    assert_eq!(batch.schema().field(1).name(), "vector_score");
    assert_eq!(batch.schema().field(2).name(), "embedding");
    assert_eq!(batch.schema().field(3).name(), "query_embedding");
}

#[test]
fn plugin_arrow_request_schema_uses_contract_columns() {
    let schema = julia_arrow_request_schema(3);

    assert_eq!(schema.field(0).name(), "doc_id");
    assert_eq!(schema.field(1).name(), "vector_score");
    assert_eq!(schema.field(2).name(), "embedding");
    assert_eq!(schema.field(3).name(), "query_embedding");
}

#[test]
fn plugin_arrow_response_schema_optionally_includes_trace_id() {
    let base = julia_arrow_response_schema(false);
    let traced = julia_arrow_response_schema(true);

    assert_eq!(base.fields().len(), 3);
    assert_eq!(traced.fields().len(), 4);
    assert_eq!(traced.field(3).name(), "trace_id");
}

#[test]
fn build_plugin_arrow_request_batch_rejects_dimension_mismatch() {
    let error = err_or_panic(
        build_plugin_arrow_request_batch(
            &[PluginArrowRequestRow {
                doc_id: "doc-1".to_string(),
                vector_score: 0.3,
                embedding: vec![1.0, 2.0],
            }],
            &[9.0, 8.0, 7.0],
        ),
        "dimension mismatch should fail",
    );

    assert!(
        error.to_string().contains("embedding dimension mismatch"),
        "unexpected error: {error}"
    );
}

#[test]
fn build_plugin_arrow_request_batch_from_embeddings_uses_candidate_ids_as_doc_ids() {
    let embeddings = BTreeMap::from([
        ("doc-1#alpha".to_string(), vec![1.0, 2.0, 3.0]),
        ("doc-2#beta".to_string(), vec![4.0, 5.0, 6.0]),
    ]);
    let batch = build_plugin_arrow_request_batch_from_embeddings(
        &[
            PluginArrowScoredCandidate {
                doc_id: "doc-1#alpha",
                vector_score: 0.31,
            },
            PluginArrowScoredCandidate {
                doc_id: "doc-2#beta",
                vector_score: 0.42,
            },
        ],
        &embeddings,
        &[9.0, 8.0, 7.0],
    )
    .unwrap_or_else(|error| panic!("request batch should build: {error}"));

    let Some(doc_ids) = batch
        .column_by_name("doc_id")
        .and_then(|column| column.as_any().downcast_ref::<StringArray>())
    else {
        panic!("doc_id column");
    };
    assert_eq!(doc_ids.value(0), "doc-1#alpha");
    assert_eq!(doc_ids.value(1), "doc-2#beta");
}

#[test]
fn build_plugin_arrow_request_batch_from_embeddings_rejects_missing_embeddings() {
    let error = err_or_panic(
        build_plugin_arrow_request_batch_from_embeddings(
            &[PluginArrowScoredCandidate {
                doc_id: "doc-1#alpha",
                vector_score: 0.31,
            }],
            &BTreeMap::new(),
            &[9.0, 8.0, 7.0],
        ),
        "missing embedding should fail",
    );

    assert!(matches!(
        error,
        PluginArrowRequestBatchBuildError::MissingEmbedding { doc_id }
        if doc_id == "doc-1#alpha"
    ));
}

#[test]
fn project_plugin_arrow_scored_candidates_collects_doc_ids_and_scores() {
    let projection =
        project_plugin_arrow_scored_candidates([("doc-1#alpha", 0.25), ("doc-2#beta", 0.5)]);

    assert_eq!(
        projection.doc_ids,
        vec!["doc-1#alpha".to_string(), "doc-2#beta".to_string()]
    );
    assert_eq!(
        projection.candidates,
        vec![
            PluginArrowScoredCandidate {
                doc_id: "doc-1#alpha",
                vector_score: 0.25,
            },
            PluginArrowScoredCandidate {
                doc_id: "doc-2#beta",
                vector_score: 0.5,
            },
        ]
    );
}

#[test]
fn build_plugin_arrow_request_batch_from_embeddings_with_metadata_sets_trace_id() {
    let batch = build_plugin_arrow_request_batch_from_embeddings_with_metadata(
        &[PluginArrowScoredCandidate {
            doc_id: "doc-1#alpha",
            vector_score: 0.25,
        }],
        &BTreeMap::from([("doc-1#alpha".to_string(), vec![1.0, 2.0, 3.0])]),
        &[9.0, 8.0, 7.0],
        "xiuxian-wendao-julia",
        "alpha signal",
        "v1",
    )
    .unwrap_or_else(|error| panic!("request batch with metadata should build: {error}"));

    assert_eq!(
        batch.schema().metadata().get("trace_id"),
        Some(&"plugin-rerank:xiuxian-wendao-julia:alpha_signal".to_string())
    );
    assert_eq!(
        batch.schema().metadata().get("wendao.schema_version"),
        Some(&"v1".to_string())
    );
}

#[tokio::test]
async fn prepare_plugin_arrow_request_rows_from_vector_store_collects_embeddings() {
    let temp_dir = tempdir_or_panic();
    let db_path = temp_dir.path().join("plugin_arrow_prepare_rows");
    let db_path_str = db_path.to_string_lossy();
    let mut store = VectorStore::new(db_path_str.as_ref(), Some(3))
        .await
        .unwrap_or_else(|error| panic!("create vector store: {error}"));
    store
        .replace_documents(
            "anchors",
            vec!["doc-1#alpha".to_string(), "doc-2#beta".to_string()],
            vec![vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0]],
            vec!["alpha".to_string(), "beta".to_string()],
            vec!["{}".to_string(), "{}".to_string()],
        )
        .await
        .unwrap_or_else(|error| panic!("seed vector table: {error}"));

    let rows = prepare_plugin_arrow_request_rows_from_vector_store(
        &store,
        "anchors",
        [
            ("doc-1#alpha".to_string(), 0.31),
            ("doc-2#beta".to_string(), 0.42),
        ],
    )
    .await
    .unwrap_or_else(|error| panic!("request rows should build: {error}"));

    assert_eq!(
        rows,
        vec![
            PluginArrowRequestRow {
                doc_id: "doc-1#alpha".to_string(),
                vector_score: 0.31,
                embedding: vec![1.0, 2.0, 3.0],
            },
            PluginArrowRequestRow {
                doc_id: "doc-2#beta".to_string(),
                vector_score: 0.42,
                embedding: vec![4.0, 5.0, 6.0],
            },
        ]
    );
}

#[tokio::test]
async fn build_plugin_arrow_request_batch_from_vector_store_with_metadata_sets_trace_id() {
    let temp_dir = tempdir_or_panic();
    let db_path = temp_dir.path().join("plugin_arrow_prepare_metadata");
    let db_path_str = db_path.to_string_lossy();
    let mut store = VectorStore::new(db_path_str.as_ref(), Some(3))
        .await
        .unwrap_or_else(|error| panic!("create vector store: {error}"));
    store
        .replace_documents(
            "anchors",
            vec!["doc-1#alpha".to_string()],
            vec![vec![1.0, 2.0, 3.0]],
            vec!["alpha".to_string()],
            vec!["{}".to_string()],
        )
        .await
        .unwrap_or_else(|error| panic!("seed vector table: {error}"));

    let batch = build_plugin_arrow_request_batch_from_vector_store_with_metadata(
        &store,
        "anchors",
        [("doc-1#alpha".to_string(), 0.25)],
        &[9.0, 8.0, 7.0],
        "xiuxian-wendao-julia",
        "alpha signal",
        "v1",
    )
    .await
    .unwrap_or_else(|error| panic!("request batch with metadata should build: {error}"));

    assert_eq!(
        batch.schema().metadata().get("trace_id"),
        Some(&"plugin-rerank:xiuxian-wendao-julia:alpha_signal".to_string())
    );
    assert_eq!(
        batch.schema().metadata().get("wendao.schema_version"),
        Some(&"v1".to_string())
    );
}

#[tokio::test]
async fn prepare_plugin_arrow_request_rows_from_vector_store_rejects_missing_embeddings() {
    let temp_dir = tempdir_or_panic();
    let db_path = temp_dir.path().join("plugin_arrow_prepare_missing");
    let db_path_str = db_path.to_string_lossy();
    let mut store = VectorStore::new(db_path_str.as_ref(), Some(3))
        .await
        .unwrap_or_else(|error| panic!("create vector store: {error}"));
    store
        .replace_documents(
            "anchors",
            vec!["doc-2#beta".to_string()],
            vec![vec![4.0, 5.0, 6.0]],
            vec!["beta".to_string()],
            vec!["{}".to_string()],
        )
        .await
        .unwrap_or_else(|error| panic!("seed vector table: {error}"));

    let error = err_or_panic(
        prepare_plugin_arrow_request_rows_from_vector_store(
            &store,
            "anchors",
            [("doc-1#alpha".to_string(), 0.31)],
        )
        .await,
        "missing embedding should fail",
    );

    assert!(matches!(
        error,
        PluginArrowVectorStoreRequestBuildError::MissingEmbedding { doc_id }
        if doc_id == "doc-1#alpha"
    ));
}

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

#[tokio::test]
async fn roundtrip_plugin_arrow_score_rows_with_binding_reports_negotiation_errors() {
    let request_batch = build_plugin_arrow_request_batch(
        &[PluginArrowRequestRow {
            doc_id: "doc-a".to_string(),
            vector_score: 0.2,
            embedding: vec![1.0, 2.0, 3.0],
        }],
        &[9.0, 8.0, 7.0],
    )
    .unwrap_or_else(|error| panic!("request batch should build: {error}"));

    let error = err_or_panic(
        roundtrip_plugin_arrow_score_rows_with_binding(
            &sample_binding(Some("not a url")),
            &request_batch,
        )
        .await,
        "invalid base_url should fail",
    );

    assert_eq!(error.selection, None);
    assert!(
        error.error.contains("invalid") || error.error.contains("URL"),
        "unexpected error: {}",
        error.error
    );
}
