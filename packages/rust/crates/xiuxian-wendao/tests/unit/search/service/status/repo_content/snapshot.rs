use crate::search::service::tests::status::helpers::ready_repo_status;
use crate::search::service::tests::status::helpers::sample_repo_documents;
use crate::search::service::tests::status::repo_content::helpers::test_service;
use crate::search::service::tests::support::{
    corpus_status, last_query_telemetry, publish_repo_bundle,
};
use crate::search::{
    SearchCorpusKind, SearchPlanePhase, SearchQueryTelemetry, SearchQueryTelemetrySource,
};

#[tokio::test]
async fn status_snapshot_reuses_last_synchronized_repo_corpus_state() {
    let service = test_service();
    let documents = sample_repo_documents();
    publish_repo_bundle(&service, "alpha/repo", &documents, Some("rev-1")).await;

    service
        .status_with_repo_content(&ready_repo_status("alpha/repo"))
        .await;

    let snapshot = service.status();
    let repo_content = corpus_status(
        &snapshot,
        SearchCorpusKind::RepoContentChunk,
        "repo content row should exist",
    );
    assert_eq!(repo_content.phase, SearchPlanePhase::Ready);
    assert!(repo_content.active_epoch.is_some());
    assert!(repo_content.row_count.unwrap_or_default() > 0);

    let repo_entity = corpus_status(
        &snapshot,
        SearchCorpusKind::RepoEntity,
        "repo entity row should exist",
    );
    assert_eq!(repo_entity.phase, SearchPlanePhase::Ready);
    assert!(repo_entity.active_epoch.is_some());
    assert!(repo_entity.row_count.unwrap_or_default() > 0);
}

#[tokio::test]
async fn status_snapshot_surfaces_last_query_telemetry() {
    let service = test_service();

    service.record_query_telemetry(
        SearchCorpusKind::KnowledgeSection,
        SearchQueryTelemetry {
            captured_at: "2026-03-23T22:20:00Z".to_string(),
            scope: None,
            source: SearchQueryTelemetrySource::Fts,
            batch_count: 3,
            rows_scanned: 120,
            matched_rows: 22,
            result_count: 10,
            batch_row_limit: Some(64),
            recall_limit_rows: Some(96),
            working_set_budget_rows: 40,
            trim_threshold_rows: 80,
            peak_working_set_rows: 55,
            trim_count: 1,
            dropped_candidate_count: 6,
        },
    );

    let snapshot = service.status();
    let knowledge = corpus_status(
        &snapshot,
        SearchCorpusKind::KnowledgeSection,
        "knowledge row should exist",
    );
    let telemetry = last_query_telemetry(knowledge, "telemetry should be present");
    assert_eq!(telemetry.captured_at, "2026-03-23T22:20:00Z");
    assert_eq!(telemetry.source, SearchQueryTelemetrySource::Fts);
    assert_eq!(telemetry.batch_count, 3);
    assert_eq!(telemetry.rows_scanned, 120);
    assert_eq!(telemetry.matched_rows, 22);
    assert_eq!(telemetry.result_count, 10);
    assert_eq!(telemetry.batch_row_limit, Some(64));
    assert_eq!(telemetry.recall_limit_rows, Some(96));
    assert_eq!(telemetry.working_set_budget_rows, 40);
    assert_eq!(telemetry.trim_threshold_rows, 80);
    assert_eq!(telemetry.peak_working_set_rows, 55);
    assert_eq!(telemetry.trim_count, 1);
    assert_eq!(telemetry.dropped_candidate_count, 6);
}
