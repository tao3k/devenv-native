use std::collections::HashSet;
use std::sync::Arc;

use crate::query_core::WendaoOperatorKind;
use crate::query_core::operators::RetrievalCorpus;
use crate::query_core::service::{RepoCodeQueryRequest, query_repo_code_relation};
use crate::query_core::telemetry::InMemoryWendaoExplainSink;
use crate::repo_index::RepoCodeDocument;
use crate::search::{SearchMaintenancePolicy, SearchManifestKeyspace, SearchPlaneService};
use crate::test_support::assert_wendao_json_snapshot;

use super::support::{
    sample_repo_analysis, sample_repo_documents, snapshot_retrieval_rows, temp_project_root,
    tempdir_or_panic,
};

#[tokio::test]
async fn query_repo_code_relation_prefers_repo_entity_corpus() {
    let temp_dir = tempdir_or_panic("tempdir");
    let service = SearchPlaneService::with_paths(
        temp_project_root(),
        temp_dir.path().join("search_plane"),
        SearchManifestKeyspace::new("xiuxian:test:query-core-repo-code-entity"),
        SearchMaintenancePolicy::default(),
    );
    service
        .publish_repo_entities_with_revision(
            "alpha/repo",
            &sample_repo_analysis("alpha/repo"),
            &sample_repo_documents(),
            Some("rev-1"),
        )
        .await
        .unwrap_or_else(|error| panic!("publish repo entities: {error}"));
    service
        .publish_repo_content_chunks_with_revision(
            "alpha/repo",
            &sample_repo_documents(),
            Some("rev-1"),
        )
        .await
        .unwrap_or_else(|error| panic!("publish repo content: {error}"));

    let telemetry = Arc::new(InMemoryWendaoExplainSink::new());
    let query = RepoCodeQueryRequest::new(
        "alpha/repo",
        "reexport",
        &HashSet::new(),
        &HashSet::new(),
        true,
        true,
        10,
    );
    let result = query_repo_code_relation(&service, &query, Some(telemetry.clone()))
        .await
        .unwrap_or_else(|error| panic!("query repo code relation: {error}"));

    assert_eq!(result.corpus, RetrievalCorpus::RepoEntity);
    assert!(result.relation.row_count() > 0);
    assert_wendao_json_snapshot(
        "query_core_repo_code_relation_prefers_repo_entity_corpus",
        serde_json::json!({
            "corpus": format!("{:?}", result.corpus),
            "rows": snapshot_retrieval_rows(&result.relation),
        }),
    );
    let events = telemetry.events();
    assert_eq!(events.len(), 3);
    assert_eq!(events[0].operator_kind, WendaoOperatorKind::VectorSearch);
    assert_eq!(events[1].operator_kind, WendaoOperatorKind::ColumnMask);
    assert_eq!(events[2].operator_kind, WendaoOperatorKind::PayloadFetch);
}

#[tokio::test]
async fn query_repo_code_relation_falls_back_to_repo_content_when_entity_lane_is_disabled() {
    let temp_dir = tempdir_or_panic("tempdir");
    let service = SearchPlaneService::with_paths(
        temp_project_root(),
        temp_dir.path().join("search_plane"),
        SearchManifestKeyspace::new("xiuxian:test:query-core-repo-code-content"),
        SearchMaintenancePolicy::default(),
    );
    service
        .publish_repo_content_chunks_with_revision(
            "alpha/repo",
            &[RepoCodeDocument {
                path: "src/BaseModelica.jl".to_string(),
                language: Some("julia".to_string()),
                contents: Arc::<str>::from(
                    "module BaseModelica\nusing Reexport\n@reexport using ModelingToolkit\nend\n",
                ),
                size_bytes: 67,
                modified_unix_ms: 0,
            }],
            Some("rev-1"),
        )
        .await
        .unwrap_or_else(|error| panic!("publish repo content: {error}"));

    let telemetry = Arc::new(InMemoryWendaoExplainSink::new());
    let query = RepoCodeQueryRequest::new(
        "alpha/repo",
        "@reexport",
        &HashSet::new(),
        &HashSet::new(),
        false,
        true,
        10,
    );
    let result = query_repo_code_relation(&service, &query, Some(telemetry.clone()))
        .await
        .unwrap_or_else(|error| panic!("query repo code relation: {error}"));

    assert_eq!(result.corpus, RetrievalCorpus::RepoContent);
    assert_eq!(result.relation.row_count(), 1);
    let rows = xiuxian_db_store::retrieval_rows_from_record_batch(&result.relation.batches()[0])
        .unwrap_or_else(|error| panic!("decode retrieval rows: {error}"));
    assert_eq!(rows[0].path, "src/BaseModelica.jl");
    assert_wendao_json_snapshot(
        "query_core_repo_code_relation_falls_back_to_repo_content",
        serde_json::json!({
            "corpus": format!("{:?}", result.corpus),
            "rows": snapshot_retrieval_rows(&result.relation),
        }),
    );
    let events = telemetry.events();
    assert_eq!(events.len(), 3);
    assert_eq!(events[2].operator_kind, WendaoOperatorKind::PayloadFetch);
}
