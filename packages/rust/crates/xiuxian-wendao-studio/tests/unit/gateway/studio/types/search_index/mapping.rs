use xiuxian_wendao::search::contracts::search_index::SearchIndexStatusResponse;
use xiuxian_wendao::search::{
    SearchCorpusKind, SearchCorpusStatus, SearchMaintenanceStatus, SearchPlanePhase,
    SearchPlaneStatusSnapshot,
};

#[test]
fn response_maps_prewarm_maintenance_metadata() {
    let mut local_symbol = SearchCorpusStatus::new(SearchCorpusKind::LocalSymbol);
    local_symbol.phase = SearchPlanePhase::Ready;
    local_symbol.maintenance = SearchMaintenanceStatus {
        prewarm_running: true,
        prewarm_queue_depth: 0,
        prewarm_queue_position: None,
        compaction_running: false,
        compaction_queue_depth: 0,
        compaction_queue_position: None,
        compaction_queue_aged: false.into(),
        compaction_pending: false,
        publish_count_since_compaction: 1,
        last_prewarmed_at: Some("2026-03-24T12:34:56Z".to_string()),
        last_prewarmed_epoch: Some(7),
        last_compacted_at: None,
        last_compaction_reason: None,
        last_compacted_row_count: None,
    };

    let response = SearchIndexStatusResponse::from(&SearchPlaneStatusSnapshot {
        repo_read_pressure: None,
        corpora: vec![local_symbol],
    });

    assert_eq!(
        response.corpora[0].maintenance.last_prewarmed_at.as_deref(),
        Some("2026-03-24T12:34:56Z")
    );
    assert!(response.corpora[0].maintenance.prewarm_running);
    assert_eq!(
        response.corpora[0].maintenance.last_prewarmed_epoch,
        Some(7)
    );
}

#[test]
fn response_maps_local_compaction_queue_metadata() {
    let mut local_symbol = SearchCorpusStatus::new(SearchCorpusKind::LocalSymbol);
    local_symbol.phase = SearchPlanePhase::Ready;
    local_symbol.maintenance = SearchMaintenanceStatus {
        prewarm_running: false,
        prewarm_queue_depth: 0,
        prewarm_queue_position: None,
        compaction_running: false,
        compaction_queue_depth: 2,
        compaction_queue_position: Some(2),
        compaction_queue_aged: true.into(),
        compaction_pending: true,
        publish_count_since_compaction: 1,
        last_prewarmed_at: None,
        last_prewarmed_epoch: None,
        last_compacted_at: None,
        last_compaction_reason: None,
        last_compacted_row_count: None,
    };

    let response = SearchIndexStatusResponse::from(&SearchPlaneStatusSnapshot {
        repo_read_pressure: None,
        corpora: vec![local_symbol],
    });

    assert_eq!(response.corpora[0].maintenance.compaction_queue_depth, 2);
    assert_eq!(
        response.corpora[0].maintenance.compaction_queue_position,
        Some(2)
    );
    assert!(response.corpora[0].maintenance.compaction_queue_aged);
}

#[test]
fn response_maps_repo_prewarm_queue_metadata() {
    let mut repo_entity = SearchCorpusStatus::new(SearchCorpusKind::RepoEntity);
    repo_entity.phase = SearchPlanePhase::Indexing;
    repo_entity.maintenance = SearchMaintenanceStatus {
        prewarm_running: false,
        prewarm_queue_depth: 1,
        prewarm_queue_position: Some(2),
        compaction_running: false,
        compaction_queue_depth: 0,
        compaction_queue_position: None,
        compaction_queue_aged: false.into(),
        compaction_pending: false,
        publish_count_since_compaction: 0,
        last_prewarmed_at: None,
        last_prewarmed_epoch: None,
        last_compacted_at: None,
        last_compaction_reason: None,
        last_compacted_row_count: None,
    };

    let response = SearchIndexStatusResponse::from(&SearchPlaneStatusSnapshot {
        repo_read_pressure: None,
        corpora: vec![repo_entity],
    });

    assert_eq!(response.corpora[0].maintenance.prewarm_queue_depth, 1);
    assert_eq!(
        response.corpora[0].maintenance.prewarm_queue_position,
        Some(2)
    );
}
