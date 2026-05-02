use std::path::PathBuf;

use crate::search::service::tests::support::{
    corpus_status, service_test_manifest_keyspace, temp_dir,
};
use crate::search::{
    SearchCorpusKind, SearchMaintenancePolicy, SearchPlanePhase, SearchPlaneService,
};

#[tokio::test]
async fn status_keeps_ready_local_corpus_out_of_compaction_reason() {
    let temp_dir = temp_dir();
    let service = SearchPlaneService::with_paths(
        PathBuf::from("/tmp/project"),
        temp_dir.path().join("search_plane"),
        service_test_manifest_keyspace(),
        SearchMaintenancePolicy {
            publish_count_threshold: 1,
            row_delta_ratio_threshold: 1.0,
        },
    );
    let lease = match service.coordinator().begin_build(
        SearchCorpusKind::LocalSymbol,
        "fp-local-ready",
        SearchCorpusKind::LocalSymbol.schema_version(),
    ) {
        crate::search::coordinator::BeginBuildDecision::Started(lease) => lease,
        other => panic!("unexpected begin result: {other:?}"),
    };

    assert!(service.publish_ready_and_maintain(&lease, 10, 3).await);

    let snapshot = service.status();
    let status = corpus_status(
        &snapshot,
        SearchCorpusKind::LocalSymbol,
        "local symbol status should exist",
    );
    assert_eq!(status.phase, SearchPlanePhase::Ready);
    assert!(!status.maintenance.compaction_running);
    assert!(!status.maintenance.compaction_pending);
    assert_eq!(status.maintenance.compaction_queue_depth, 0);
    assert_eq!(status.maintenance.compaction_queue_position, None);
    assert!(!status.maintenance.compaction_queue_aged);
    assert!(status.status_reason.is_none());
}
