use std::path::PathBuf;

use crate::search::service::tests::support::{
    assert_status_reason, corpus_status, service_test_manifest_keyspace, temp_dir,
};
use crate::search::{
    SearchCorpusKind, SearchCorpusStatusAction, SearchCorpusStatusReasonCode,
    SearchCorpusStatusSeverity, SearchMaintenancePolicy, SearchPlanePhase, SearchPlaneService,
};

#[test]
fn status_marks_indexing_corpus_with_running_prewarm_reason() {
    let temp_dir = temp_dir();
    let service = SearchPlaneService::with_paths(
        PathBuf::from("/tmp/project"),
        temp_dir.path().join("search_plane"),
        service_test_manifest_keyspace(),
        SearchMaintenancePolicy::default(),
    );
    let lease = match service.coordinator().begin_build(
        SearchCorpusKind::LocalSymbol,
        "fp-prewarm-running",
        SearchCorpusKind::LocalSymbol.schema_version(),
    ) {
        crate::search::coordinator::BeginBuildDecision::Started(lease) => lease,
        other => panic!("unexpected begin result: {other:?}"),
    };

    assert!(
        service
            .coordinator()
            .mark_prewarm_running(SearchCorpusKind::LocalSymbol, lease.epoch)
    );

    let snapshot = service.status();
    let status = corpus_status(
        &snapshot,
        SearchCorpusKind::LocalSymbol,
        "local symbol status should exist",
    );
    assert_eq!(status.phase, SearchPlanePhase::Indexing);
    assert_eq!(status.staging_epoch, Some(lease.epoch));
    assert!(status.maintenance.prewarm_running);
    assert_status_reason(
        status,
        SearchCorpusStatusReasonCode::Prewarming,
        SearchCorpusStatusSeverity::Info,
        SearchCorpusStatusAction::Wait,
        false,
    );
}

#[test]
fn status_marks_indexing_corpus_with_prewarmed_staging_reason() {
    let temp_dir = temp_dir();
    let service = SearchPlaneService::with_paths(
        PathBuf::from("/tmp/project"),
        temp_dir.path().join("search_plane"),
        service_test_manifest_keyspace(),
        SearchMaintenancePolicy::default(),
    );
    let lease = match service.coordinator().begin_build(
        SearchCorpusKind::LocalSymbol,
        "fp-prewarmed-staging",
        SearchCorpusKind::LocalSymbol.schema_version(),
    ) {
        crate::search::coordinator::BeginBuildDecision::Started(lease) => lease,
        other => panic!("unexpected begin result: {other:?}"),
    };

    assert!(
        service
            .coordinator()
            .mark_prewarm_complete(SearchCorpusKind::LocalSymbol, lease.epoch)
    );

    let snapshot = service.status();
    let status = corpus_status(
        &snapshot,
        SearchCorpusKind::LocalSymbol,
        "local symbol status should exist",
    );
    assert_eq!(status.phase, SearchPlanePhase::Indexing);
    assert_eq!(status.staging_epoch, Some(lease.epoch));
    assert!(!status.maintenance.prewarm_running);
    assert_eq!(status.maintenance.last_prewarmed_epoch, Some(lease.epoch));
    assert!(status.maintenance.last_prewarmed_at.is_some());
    assert_status_reason(
        status,
        SearchCorpusStatusReasonCode::Prewarming,
        SearchCorpusStatusSeverity::Info,
        SearchCorpusStatusAction::Wait,
        false,
    );
}
