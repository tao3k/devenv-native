use std::collections::{BTreeSet, HashSet};
use std::sync::Arc;

use crate::query_core::WendaoRelation;
use crate::query_core::context::WendaoExecutionContext;
use crate::query_core::execute::{
    SearchPlaneRetrievalBackend, execute_column_mask, execute_graph_neighbors,
    execute_payload_fetch, execute_vector_search,
};
use crate::query_core::operators::{
    ColumnMaskOp, ColumnMaskPredicate, GraphDirection, GraphNeighborsOp, PayloadFetchOp,
    RetrievalCorpus, VectorSearchOp,
};
use crate::query_core::telemetry::InMemoryWendaoExplainSink;
use crate::search::{SearchMaintenancePolicy, SearchManifestKeyspace, SearchPlaneService};

use super::support::{
    StubGraphBackend, StubPayloadRetrievalBackend, repo_document, temp_project_root,
    tempdir_or_panic,
};

#[tokio::test]
async fn vector_search_routes_through_search_plane_adapter_and_returns_relation() {
    let temp_dir = tempdir_or_panic("tempdir");
    let service = Arc::new(SearchPlaneService::with_paths(
        temp_project_root(),
        temp_dir.path().join("search_plane"),
        SearchManifestKeyspace::new("xiuxian:test:query-core"),
        SearchMaintenancePolicy::default(),
    ));
    service
        .publish_repo_content_chunks_with_revision(
            "alpha/repo",
            &[repo_document("src/lib.rs", "fn alpha() {}\n", 14, 10)],
            Some("rev-1"),
        )
        .await
        .unwrap_or_else(|error| panic!("publish repo content: {error}"));

    let telemetry = Arc::new(InMemoryWendaoExplainSink::new());
    let ctx = WendaoExecutionContext::default()
        .with_retrieval_backend(Arc::new(SearchPlaneRetrievalBackend::new(Arc::clone(
            &service,
        ))))
        .with_explain_sink(telemetry.clone());
    let relation = execute_vector_search(
        &ctx,
        &VectorSearchOp {
            corpus: RetrievalCorpus::RepoContent,
            repo_id: "alpha/repo".to_string(),
            search_term: "alpha".to_string(),
            language_filters: HashSet::new(),
            kind_filters: HashSet::new(),
            limit: 5,
        },
    )
    .await
    .unwrap_or_else(|error| panic!("execute vector search: {error}"));

    assert_eq!(relation.row_count(), 1);
    let events = telemetry.events();
    assert_eq!(events.len(), 1);
    assert!(events[0].legacy_adapter);
}

#[tokio::test]
async fn graph_neighbors_routes_through_link_graph_adapter_and_returns_relation() {
    let batch = arrow::record_batch::RecordBatch::try_new(
        Arc::new(arrow::datatypes::Schema::new(vec![
            arrow::datatypes::Field::new("node_id", arrow::datatypes::DataType::Utf8, false),
            arrow::datatypes::Field::new("path", arrow::datatypes::DataType::Utf8, false),
            arrow::datatypes::Field::new("title", arrow::datatypes::DataType::Utf8, true),
            arrow::datatypes::Field::new("distance", arrow::datatypes::DataType::UInt64, false),
            arrow::datatypes::Field::new("direction", arrow::datatypes::DataType::Utf8, false),
        ])),
        vec![
            Arc::new(arrow::array::StringArray::from(vec!["alpha", "beta"]))
                as arrow::array::ArrayRef,
            Arc::new(arrow::array::StringArray::from(vec!["alpha.md", "beta.md"]))
                as arrow::array::ArrayRef,
            Arc::new(arrow::array::StringArray::from(vec![
                Some("Alpha"),
                Some("Beta"),
            ])) as arrow::array::ArrayRef,
            Arc::new(arrow::array::UInt64Array::from(vec![0, 1])) as arrow::array::ArrayRef,
            Arc::new(arrow::array::StringArray::from(vec!["center", "both"]))
                as arrow::array::ArrayRef,
        ],
    )
    .unwrap_or_else(|error| panic!("graph batch: {error}"));
    let relation = WendaoRelation::new(batch.schema(), vec![batch]);
    let telemetry = Arc::new(InMemoryWendaoExplainSink::new());
    let ctx = WendaoExecutionContext::default()
        .with_graph_backend(Arc::new(StubGraphBackend { relation }))
        .with_explain_sink(telemetry.clone());

    let relation = execute_graph_neighbors(
        &ctx,
        &GraphNeighborsOp {
            node_id: "alpha.md".to_string(),
            direction: GraphDirection::Both,
            hops: 1,
            limit: 10,
        },
    )
    .await
    .unwrap_or_else(|error| panic!("graph neighbors: {error}"));

    assert!(relation.row_count() >= 2);
    let events = telemetry.events();
    assert_eq!(events.len(), 1);
    assert!(events[0].legacy_adapter);
}

#[tokio::test]
async fn column_mask_filters_before_payload_fetch_and_emits_phase_counts() {
    let temp_dir = tempdir_or_panic("tempdir");
    let service = Arc::new(SearchPlaneService::with_paths(
        temp_project_root(),
        temp_dir.path().join("search_plane"),
        SearchManifestKeyspace::new("xiuxian:test:query-core-mask"),
        SearchMaintenancePolicy::default(),
    ));
    service
        .publish_repo_content_chunks_with_revision(
            "alpha/repo",
            &[
                repo_document("src/lib.rs", "fn alpha() {}\n", 14, 10),
                repo_document("src/util.rs", "fn beta() {}\n", 13, 10),
            ],
            Some("rev-1"),
        )
        .await
        .unwrap_or_else(|error| panic!("publish repo content: {error}"));

    let telemetry = Arc::new(InMemoryWendaoExplainSink::new());
    let ctx = WendaoExecutionContext::default()
        .with_retrieval_backend(Arc::new(SearchPlaneRetrievalBackend::new(Arc::clone(
            &service,
        ))))
        .with_explain_sink(telemetry.clone());
    let relation = execute_vector_search(
        &ctx,
        &VectorSearchOp {
            corpus: RetrievalCorpus::RepoContent,
            repo_id: "alpha/repo".to_string(),
            search_term: "fn".to_string(),
            language_filters: HashSet::new(),
            kind_filters: HashSet::new(),
            limit: 10,
        },
    )
    .await
    .unwrap_or_else(|error| panic!("execute vector search: {error}"));

    let masked = execute_column_mask(
        &ctx,
        &ColumnMaskOp {
            relation,
            predicates: vec![ColumnMaskPredicate::PathContains("util".to_string())],
            limit: Some(1),
        },
    )
    .unwrap_or_else(|error| panic!("column mask: {error}"));
    assert_eq!(masked.row_count(), 1);

    let fetched = execute_payload_fetch(
        &ctx,
        &PayloadFetchOp {
            relation: masked,
            columns: vec!["id".to_string(), "path".to_string()],
            ids: Some(BTreeSet::from(["src/util.rs".to_string()])),
        },
    )
    .await
    .unwrap_or_else(|error| panic!("payload fetch: {error}"));
    assert_eq!(fetched.row_count(), 0);

    let events = telemetry.events();
    assert_eq!(events.len(), 3);
    assert_eq!(events[1].narrow_phase_surviving_count, Some(1));
    assert_eq!(events[2].payload_phase_fetched_count, Some(0));
}

#[tokio::test]
async fn payload_fetch_projects_requested_columns() {
    let telemetry = Arc::new(InMemoryWendaoExplainSink::new());
    let ctx = WendaoExecutionContext::default().with_explain_sink(telemetry.clone());
    let batch =
        xiuxian_db_store::retrieval_rows_to_record_batch(&[xiuxian_db_store::RetrievalRow {
            id: "alpha".to_string(),
            path: "src/lib.rs".to_string(),
            repo: Some("alpha/repo".to_string()),
            title: Some("Alpha".to_string()),
            score: Some(0.9),
            source: "test".to_string(),
            snippet: Some("fn alpha()".to_string()),
            doc_type: Some("file".into()),
            match_reason: Some("repo_content_search".to_string()),
            best_section: Some("3: fn alpha()".to_string()),
            language: Some("rust".to_string()),
            line: Some(3),
        }])
        .unwrap_or_else(|error| panic!("build retrieval batch: {error}"));
    let relation = WendaoRelation::new(batch.schema(), vec![batch]);
    let backend = Arc::new(StubPayloadRetrievalBackend);
    let ctx = ctx.with_retrieval_backend(backend);

    let fetched = execute_payload_fetch(
        &ctx,
        &PayloadFetchOp {
            relation,
            columns: vec!["id".to_string(), "path".to_string()],
            ids: None,
        },
    )
    .await
    .unwrap_or_else(|error| panic!("payload fetch: {error}"));
    let field_names = fetched
        .schema()
        .fields()
        .iter()
        .map(|field| field.name().as_str())
        .collect::<Vec<_>>();
    assert_eq!(field_names, vec!["id", "path"]);
}
