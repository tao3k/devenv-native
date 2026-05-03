#[cfg(feature = "duckdb")]
use xiuxian_wendao::duckdb::LocalRelationEngineKind;
use xiuxian_wendao::search::contracts::search_index::SearchIndexStatusResponse;
#[cfg(feature = "duckdb")]
use xiuxian_wendao::search::contracts::search_index::configured_status_diagnostics_engine_kind;
use xiuxian_wendao::search::SearchPlaneStatusSnapshot;
#[cfg(feature = "duckdb")]
use xiuxian_wendao::set_link_graph_wendao_config_override;

use super::helpers::{
    compacting_local_symbol_status, degraded_repo_entity_status, telemetry_attachment_status,
    telemetry_knowledge_status,
};

fn sample_status_snapshot() -> SearchPlaneStatusSnapshot {
    SearchPlaneStatusSnapshot {
        repo_read_pressure: None,
        corpora: vec![
            compacting_local_symbol_status(),
            degraded_repo_entity_status(),
            telemetry_attachment_status(),
            telemetry_knowledge_status(),
        ],
    }
}

fn sample_scope_bucketed_snapshot() -> SearchPlaneStatusSnapshot {
    let mut local_symbol =
        xiuxian_wendao::search::SearchCorpusStatus::new(xiuxian_wendao::search::SearchCorpusKind::LocalSymbol);
    local_symbol.phase = xiuxian_wendao::search::SearchPlanePhase::Ready;
    local_symbol.last_query_telemetry = Some(xiuxian_wendao::search::SearchQueryTelemetry {
        captured_at: "2026-03-23T22:10:00Z".to_string(),
        scope: Some("autocomplete".to_string()),
        source: xiuxian_wendao::search::SearchQueryTelemetrySource::Scan,
        batch_count: 2,
        rows_scanned: 25,
        matched_rows: 9,
        result_count: 5,
        batch_row_limit: Some(16),
        recall_limit_rows: Some(32),
        working_set_budget_rows: 12,
        trim_threshold_rows: 24,
        peak_working_set_rows: 14,
        trim_count: 1,
        dropped_candidate_count: 3,
    });

    let mut reference = xiuxian_wendao::search::SearchCorpusStatus::new(
        xiuxian_wendao::search::SearchCorpusKind::ReferenceOccurrence,
    );
    reference.phase = xiuxian_wendao::search::SearchPlanePhase::Ready;
    reference.last_query_telemetry = Some(xiuxian_wendao::search::SearchQueryTelemetry {
        captured_at: "2026-03-23T22:11:00Z".to_string(),
        scope: Some("search".to_string()),
        source: xiuxian_wendao::search::SearchQueryTelemetrySource::Fts,
        batch_count: 3,
        rows_scanned: 40,
        matched_rows: 12,
        result_count: 6,
        batch_row_limit: Some(24),
        recall_limit_rows: Some(48),
        working_set_budget_rows: 18,
        trim_threshold_rows: 36,
        peak_working_set_rows: 21,
        trim_count: 0,
        dropped_candidate_count: 0,
    });

    let mut attachment =
        xiuxian_wendao::search::SearchCorpusStatus::new(xiuxian_wendao::search::SearchCorpusKind::Attachment);
    attachment.phase = xiuxian_wendao::search::SearchPlanePhase::Ready;
    attachment.last_query_telemetry = Some(xiuxian_wendao::search::SearchQueryTelemetry {
        captured_at: "2026-03-23T22:12:00Z".to_string(),
        scope: Some("search".to_string()),
        source: xiuxian_wendao::search::SearchQueryTelemetrySource::FtsFallbackScan,
        batch_count: 4,
        rows_scanned: 60,
        matched_rows: 15,
        result_count: 7,
        batch_row_limit: Some(32),
        recall_limit_rows: Some(64),
        working_set_budget_rows: 24,
        trim_threshold_rows: 48,
        peak_working_set_rows: 29,
        trim_count: 2,
        dropped_candidate_count: 5,
    });

    SearchPlaneStatusSnapshot {
        repo_read_pressure: None,
        corpora: vec![local_symbol, reference, attachment],
    }
}

fn sample_repo_read_pressure_snapshot() -> SearchPlaneStatusSnapshot {
    SearchPlaneStatusSnapshot {
        repo_read_pressure: Some(xiuxian_wendao::search::SearchRepoReadPressure {
            budget: 8,
            in_flight: 3,
            captured_at: Some("2026-04-09T19:10:00Z".to_string()),
            requested_repo_count: Some(24),
            searchable_repo_count: Some(9),
            parallelism: Some(6),
            fanout_capped: true,
        }),
        corpora: vec![
            compacting_local_symbol_status(),
            degraded_repo_entity_status(),
            telemetry_attachment_status(),
        ],
    }
}

#[tokio::test]
async fn diagnostics_rollup_matches_rust_status_response() {
    let snapshot = sample_status_snapshot();

    let response = SearchIndexStatusResponse::from_snapshot_with_diagnostics(&snapshot).await;
    let baseline = SearchIndexStatusResponse::from(&snapshot);

    assert_eq!(response, baseline);
}

#[tokio::test]
async fn diagnostics_rollup_matches_rust_status_response_for_scope_bucketed_telemetry() {
    let snapshot = sample_scope_bucketed_snapshot();

    let response = SearchIndexStatusResponse::from_snapshot_with_diagnostics(&snapshot).await;
    let baseline = SearchIndexStatusResponse::from(&snapshot);

    assert_eq!(response, baseline);
}

#[tokio::test]
async fn diagnostics_rollup_matches_rust_status_response_for_repo_read_pressure() {
    let snapshot = sample_repo_read_pressure_snapshot();

    let response = SearchIndexStatusResponse::from_snapshot_with_diagnostics(&snapshot).await;
    let baseline = SearchIndexStatusResponse::from(&snapshot);

    assert_eq!(response, baseline);
    let repo_read_pressure = response
        .repo_read_pressure
        .as_ref()
        .unwrap_or_else(|| panic!("repo read pressure should be present"));
    assert_eq!(repo_read_pressure.budget, 8);
    assert_eq!(repo_read_pressure.in_flight, 3);
    assert_eq!(
        repo_read_pressure.captured_at.as_deref(),
        Some("2026-04-09T19:10:00Z")
    );
    assert_eq!(repo_read_pressure.requested_repo_count, Some(24));
    assert_eq!(repo_read_pressure.searchable_repo_count, Some(9));
    assert_eq!(repo_read_pressure.parallelism, Some(6));
    assert!(repo_read_pressure.fanout_capped);
}

#[cfg(feature = "duckdb")]
#[tokio::test]
async fn diagnostics_rollup_selects_duckdb_when_enabled() {
    let temp = tempfile::tempdir().unwrap_or_else(|error| panic!("temp dir: {error}"));
    let config_path = temp.path().join("wendao.toml");
    std::fs::write(
        &config_path,
        r#"[search.duckdb]
enabled = true
database_path = ":memory:"
temp_directory = ".data/duckdb/tmp"
threads = 2
materialize_threshold_rows = 16
prefer_virtual_arrow = true
"#,
    )
    .unwrap_or_else(|error| panic!("write config: {error}"));
    set_link_graph_wendao_config_override(&config_path.to_string_lossy());

    let snapshot = sample_status_snapshot();
    let response = SearchIndexStatusResponse::from_snapshot_with_diagnostics(&snapshot).await;
    let baseline = SearchIndexStatusResponse::from(&snapshot);

    assert_eq!(
        configured_status_diagnostics_engine_kind().unwrap_or(LocalRelationEngineKind::DataFusion),
        LocalRelationEngineKind::DuckDb
    );
    assert_eq!(response, baseline);
}
