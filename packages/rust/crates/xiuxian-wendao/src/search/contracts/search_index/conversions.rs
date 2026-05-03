use super::definitions as search_index;

impl From<crate::search::SearchPlanePhase> for search_index::SearchIndexPhase {
    fn from(value: crate::search::SearchPlanePhase) -> Self {
        match value {
            crate::search::SearchPlanePhase::Idle => Self::Idle,
            crate::search::SearchPlanePhase::Indexing => Self::Indexing,
            crate::search::SearchPlanePhase::Ready => Self::Ready,
            crate::search::SearchPlanePhase::Degraded => Self::Degraded,
            crate::search::SearchPlanePhase::Failed => Self::Failed,
        }
    }
}

impl From<crate::search::SearchCorpusIssueCode> for search_index::SearchIndexIssueCode {
    fn from(value: crate::search::SearchCorpusIssueCode) -> Self {
        match value {
            crate::search::SearchCorpusIssueCode::PublishedManifestMissing => {
                Self::PublishedManifestMissing
            }
            crate::search::SearchCorpusIssueCode::PublishedRevisionMissing => {
                Self::PublishedRevisionMissing
            }
            crate::search::SearchCorpusIssueCode::PublishedRevisionMismatch => {
                Self::PublishedRevisionMismatch
            }
            crate::search::SearchCorpusIssueCode::RepoIndexFailed => Self::RepoIndexFailed,
        }
    }
}

impl From<crate::search::SearchCorpusIssueFamily> for search_index::SearchIndexIssueFamily {
    fn from(value: crate::search::SearchCorpusIssueFamily) -> Self {
        match value {
            crate::search::SearchCorpusIssueFamily::Manifest => Self::Manifest,
            crate::search::SearchCorpusIssueFamily::Revision => Self::Revision,
            crate::search::SearchCorpusIssueFamily::RepoSync => Self::RepoSync,
            crate::search::SearchCorpusIssueFamily::Mixed => Self::Mixed,
        }
    }
}

impl From<&crate::search::SearchCorpusIssue> for search_index::SearchIndexIssue {
    fn from(value: &crate::search::SearchCorpusIssue) -> Self {
        Self {
            code: value.code.into(),
            readable: value.readable,
            repo_id: value.repo_id.clone(),
            current_revision: value.current_revision.clone(),
            published_revision: value.published_revision.clone(),
            message: value.message.clone(),
        }
    }
}

impl From<&crate::search::SearchCorpusIssueSummary> for search_index::SearchIndexIssueSummary {
    fn from(value: &crate::search::SearchCorpusIssueSummary) -> Self {
        Self {
            family: value.family.into(),
            primary_code: value.primary_code.into(),
            issue_count: value.issue_count,
            readable_issue_count: value.readable_issue_count,
        }
    }
}

impl From<crate::search::SearchCorpusStatusSeverity> for search_index::SearchIndexStatusSeverity {
    fn from(value: crate::search::SearchCorpusStatusSeverity) -> Self {
        match value {
            crate::search::SearchCorpusStatusSeverity::Info => Self::Info,
            crate::search::SearchCorpusStatusSeverity::Warning => Self::Warning,
            crate::search::SearchCorpusStatusSeverity::Error => Self::Error,
        }
    }
}

impl From<crate::search::SearchCorpusStatusAction> for search_index::SearchIndexStatusAction {
    fn from(value: crate::search::SearchCorpusStatusAction) -> Self {
        match value {
            crate::search::SearchCorpusStatusAction::Wait => Self::Wait,
            crate::search::SearchCorpusStatusAction::RetryBuild => Self::RetryBuild,
            crate::search::SearchCorpusStatusAction::ResyncRepo => Self::ResyncRepo,
            crate::search::SearchCorpusStatusAction::InspectRepoSync => Self::InspectRepoSync,
        }
    }
}

impl From<crate::search::SearchCorpusStatusReasonCode>
    for search_index::SearchIndexStatusReasonCode
{
    fn from(value: crate::search::SearchCorpusStatusReasonCode) -> Self {
        match value {
            crate::search::SearchCorpusStatusReasonCode::WarmingUp => Self::WarmingUp,
            crate::search::SearchCorpusStatusReasonCode::Prewarming => Self::Prewarming,
            crate::search::SearchCorpusStatusReasonCode::Refreshing => Self::Refreshing,
            crate::search::SearchCorpusStatusReasonCode::Compacting => Self::Compacting,
            crate::search::SearchCorpusStatusReasonCode::CompactionPending => {
                Self::CompactionPending
            }
            crate::search::SearchCorpusStatusReasonCode::BuildFailed => Self::BuildFailed,
            crate::search::SearchCorpusStatusReasonCode::PublishedManifestMissing => {
                Self::PublishedManifestMissing
            }
            crate::search::SearchCorpusStatusReasonCode::PublishedRevisionMissing => {
                Self::PublishedRevisionMissing
            }
            crate::search::SearchCorpusStatusReasonCode::PublishedRevisionMismatch => {
                Self::PublishedRevisionMismatch
            }
            crate::search::SearchCorpusStatusReasonCode::RepoIndexFailed => Self::RepoIndexFailed,
        }
    }
}

impl From<&crate::search::SearchCorpusStatusReason> for search_index::SearchIndexStatusReason {
    fn from(value: &crate::search::SearchCorpusStatusReason) -> Self {
        Self {
            code: value.code.into(),
            severity: value.severity.into(),
            action: value.action.into(),
            readable: value.readable,
        }
    }
}

impl From<&crate::search::SearchMaintenanceStatus> for search_index::SearchIndexMaintenanceStatus {
    fn from(value: &crate::search::SearchMaintenanceStatus) -> Self {
        Self {
            prewarm_running: value.prewarm_running,
            prewarm_queue_depth: value.prewarm_queue_depth,
            prewarm_queue_position: value.prewarm_queue_position,
            compaction_running: value.compaction_running,
            compaction_queue_depth: value.compaction_queue_depth,
            compaction_queue_position: value.compaction_queue_position,
            compaction_queue_aged: value.compaction_queue_aged.is_aged().into(),
            compaction_pending: value.compaction_pending,
            publish_count_since_compaction: value.publish_count_since_compaction,
            last_prewarmed_at: value.last_prewarmed_at.clone(),
            last_prewarmed_epoch: value.last_prewarmed_epoch,
            last_compacted_at: value.last_compacted_at.clone(),
            last_compaction_reason: value.last_compaction_reason.clone(),
            last_compacted_row_count: value.last_compacted_row_count,
        }
    }
}

impl From<&crate::search::SearchRepoReadPressure> for search_index::SearchIndexRepoReadPressure {
    fn from(value: &crate::search::SearchRepoReadPressure) -> Self {
        Self {
            budget: value.budget,
            in_flight: value.in_flight,
            captured_at: value.captured_at.clone(),
            requested_repo_count: value.requested_repo_count,
            searchable_repo_count: value.searchable_repo_count,
            parallelism: value.parallelism,
            fanout_capped: value.fanout_capped,
        }
    }
}

impl From<crate::search::SearchQueryTelemetrySource>
    for search_index::SearchIndexQueryTelemetrySource
{
    fn from(value: crate::search::SearchQueryTelemetrySource) -> Self {
        match value {
            crate::search::SearchQueryTelemetrySource::Scan => Self::Scan,
            crate::search::SearchQueryTelemetrySource::Fts => Self::Fts,
            crate::search::SearchQueryTelemetrySource::FtsFallbackScan => Self::FtsFallbackScan,
        }
    }
}

impl From<&crate::search::SearchQueryTelemetry> for search_index::SearchIndexQueryTelemetry {
    fn from(value: &crate::search::SearchQueryTelemetry) -> Self {
        Self {
            captured_at: value.captured_at.clone(),
            scope: value.scope.clone(),
            source: value.source.into(),
            batch_count: value.batch_count,
            rows_scanned: value.rows_scanned,
            matched_rows: value.matched_rows,
            result_count: value.result_count,
            batch_row_limit: value.batch_row_limit,
            recall_limit_rows: value.recall_limit_rows,
            working_set_budget_rows: value.working_set_budget_rows,
            trim_threshold_rows: value.trim_threshold_rows,
            peak_working_set_rows: value.peak_working_set_rows,
            trim_count: value.trim_count,
            dropped_candidate_count: value.dropped_candidate_count,
        }
    }
}

impl From<&crate::search::SearchCorpusStatus> for search_index::SearchCorpusIndexStatus {
    fn from(value: &crate::search::SearchCorpusStatus) -> Self {
        Self {
            corpus: value.corpus.to_string(),
            phase: value.phase.into(),
            active_epoch: value.active_epoch,
            staging_epoch: value.staging_epoch,
            schema_version: value.schema_version,
            fingerprint: value.fingerprint.clone(),
            progress: value.progress,
            row_count: value.row_count,
            fragment_count: value.fragment_count,
            build_started_at: value.build_started_at.clone(),
            build_finished_at: value.build_finished_at.clone(),
            updated_at: value.updated_at.clone(),
            last_error: value.last_error.clone(),
            issues: value
                .issues
                .iter()
                .map(search_index::SearchIndexIssue::from)
                .collect(),
            issue_summary: value
                .issue_summary
                .as_ref()
                .map(search_index::SearchIndexIssueSummary::from),
            status_reason: value
                .status_reason
                .as_ref()
                .map(search_index::SearchIndexStatusReason::from),
            last_query_telemetry: value
                .last_query_telemetry
                .as_ref()
                .map(search_index::SearchIndexQueryTelemetry::from),
            maintenance: search_index::SearchIndexMaintenanceStatus::from(&value.maintenance),
        }
    }
}
