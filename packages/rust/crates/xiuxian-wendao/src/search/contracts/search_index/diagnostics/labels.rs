use crate::search::SearchQueryTelemetrySource;
use crate::search::contracts::search_index::definitions as search_index;

pub(super) fn query_telemetry_source_label(source: SearchQueryTelemetrySource) -> &'static str {
    match source {
        SearchQueryTelemetrySource::Scan => "scan",
        SearchQueryTelemetrySource::Fts => "fts",
        SearchQueryTelemetrySource::FtsFallbackScan => "fts_fallback_scan",
    }
}

pub(super) fn status_reason_code_label(
    code: search_index::SearchIndexStatusReasonCode,
) -> &'static str {
    match code {
        search_index::SearchIndexStatusReasonCode::WarmingUp => "warming_up",
        search_index::SearchIndexStatusReasonCode::Prewarming => "prewarming",
        search_index::SearchIndexStatusReasonCode::Refreshing => "refreshing",
        search_index::SearchIndexStatusReasonCode::Compacting => "compacting",
        search_index::SearchIndexStatusReasonCode::CompactionPending => "compaction_pending",
        search_index::SearchIndexStatusReasonCode::BuildFailed => "build_failed",
        search_index::SearchIndexStatusReasonCode::PublishedManifestMissing => {
            "published_manifest_missing"
        }
        search_index::SearchIndexStatusReasonCode::PublishedRevisionMissing => {
            "published_revision_missing"
        }
        search_index::SearchIndexStatusReasonCode::PublishedRevisionMismatch => {
            "published_revision_mismatch"
        }
        search_index::SearchIndexStatusReasonCode::RepoIndexFailed => "repo_index_failed",
    }
}

pub(super) fn status_reason_severity_label(
    severity: search_index::SearchIndexStatusSeverity,
) -> &'static str {
    match severity {
        search_index::SearchIndexStatusSeverity::Info => "info",
        search_index::SearchIndexStatusSeverity::Warning => "warning",
        search_index::SearchIndexStatusSeverity::Error => "error",
    }
}

pub(super) fn status_reason_action_label(
    action: search_index::SearchIndexStatusAction,
) -> &'static str {
    match action {
        search_index::SearchIndexStatusAction::Wait => "wait",
        search_index::SearchIndexStatusAction::RetryBuild => "retry_build",
        search_index::SearchIndexStatusAction::ResyncRepo => "resync_repo",
        search_index::SearchIndexStatusAction::InspectRepoSync => "inspect_repo_sync",
    }
}

pub(super) fn bounded_u64_to_i64(value: u64) -> i64 {
    i64::try_from(value).unwrap_or(i64::MAX)
}
