mod assertions;

use xiuxian_wendao::search::contracts::search_index::SearchIndexStatusResponse;
use xiuxian_wendao::search::SearchPlaneStatusSnapshot;

use super::helpers::{
    compacting_local_symbol_status, degraded_repo_entity_status, telemetry_attachment_status,
    telemetry_knowledge_status,
};
use assertions::{
    assert_aggregate_reason, assert_attachment_telemetry, assert_local_compaction_status,
    assert_maintenance_summary, assert_repo_issue_rollup, assert_response_counts,
    assert_telemetry_summary,
};

#[test]
fn response_counts_track_phase_and_compaction_state() {
    let response = SearchIndexStatusResponse::from(&SearchPlaneStatusSnapshot {
        repo_read_pressure: None,
        corpora: vec![
            compacting_local_symbol_status(),
            degraded_repo_entity_status(),
            telemetry_attachment_status(),
            telemetry_knowledge_status(),
        ],
    });

    assert_response_counts(&response);
    assert_aggregate_reason(&response);
    assert_maintenance_summary(&response);
    assert_local_compaction_status(&response);
    assert_repo_issue_rollup(&response);
    assert_attachment_telemetry(&response);
    assert_telemetry_summary(&response);
}
