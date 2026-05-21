//! Compatibility path boundary: this module preserves an established Wendao owner path while the API surface is being narrowed.
use crate::repo_index::RepoIndexEntryStatus;
use crate::search::{
    SearchCorpusIssue, SearchCorpusStatus, SearchMaintenanceStatus, SearchRepoPublicationRecord,
};

pub(super) type PublishedRepoTable = (Option<RepoIndexEntryStatus>, SearchRepoPublicationRecord);

pub(super) struct RepoTableStatusSynthesis {
    pub(super) status: SearchCorpusStatus,
    pub(super) published_repos: Vec<PublishedRepoTable>,
    pub(super) issues: Vec<SearchCorpusIssue>,
    pub(super) has_active_work: bool,
    pub(super) runtime_statuses: Vec<RepoIndexEntryStatus>,
}

#[derive(Default)]
pub(super) struct RepoTableSummary {
    pub(super) has_ready_tables: bool,
    pub(super) row_count: u64,
    pub(super) fragment_count: u64,
    pub(super) publication_epochs: Vec<u64>,
    pub(super) fingerprint_parts: Vec<String>,
    pub(super) fingerprint: String,
    pub(super) build_finished_at: Option<String>,
    pub(super) updated_at: Option<String>,
}

pub(super) struct RepoPrewarmRuntimeView {
    pub(super) is_running: bool,
    pub(super) queue_depth: u32,
    pub(super) queue_position: Option<u32>,
}

pub(super) struct RepoCompactionRuntimeView {
    pub(super) is_running: bool,
    pub(super) queue_depth: u32,
    pub(super) queue_position: Option<u32>,
    pub(super) queue_aged: bool,
}

pub(super) fn merge_repo_maintenance(
    target: &mut SearchMaintenanceStatus,
    source: &SearchMaintenanceStatus,
) {
    target.prewarm_running |= source.prewarm_running;
    target.prewarm_queue_depth = target.prewarm_queue_depth.max(source.prewarm_queue_depth);
    match (target.prewarm_queue_position, source.prewarm_queue_position) {
        (None, Some(source_position)) => {
            target.prewarm_queue_position = Some(source_position);
        }
        (Some(target_position), Some(source_position)) if source_position < target_position => {
            target.prewarm_queue_position = Some(source_position);
        }
        _ => {}
    }
    target.compaction_running |= source.compaction_running;
    target.compaction_queue_aged |= source.compaction_queue_aged;
    target.compaction_queue_depth = target
        .compaction_queue_depth
        .max(source.compaction_queue_depth);
    match (
        target.compaction_queue_position,
        source.compaction_queue_position,
    ) {
        (None, Some(source_position)) => {
            target.compaction_queue_position = Some(source_position);
        }
        (Some(target_position), Some(source_position)) if source_position < target_position => {
            target.compaction_queue_position = Some(source_position);
        }
        _ => {}
    }
    target.compaction_pending |= source.compaction_pending;
    target.publish_count_since_compaction = target
        .publish_count_since_compaction
        .max(source.publish_count_since_compaction);
    merge_latest_epoch_timestamp(
        &mut target.last_prewarmed_at,
        &mut target.last_prewarmed_epoch,
        source.last_prewarmed_at.as_deref(),
        source.last_prewarmed_epoch,
    );
    merge_latest_reason_timestamp(
        &mut target.last_compacted_at,
        &mut target.last_compaction_reason,
        source.last_compacted_at.as_deref(),
        source.last_compaction_reason.as_deref(),
    );
    match (
        target.last_compacted_row_count,
        source.last_compacted_row_count,
        target.last_compacted_at.as_deref(),
        source.last_compacted_at.as_deref(),
    ) {
        (None, Some(source_rows), _, _) => {
            target.last_compacted_row_count = Some(source_rows);
        }
        (Some(_), Some(source_rows), Some(target_timestamp), Some(source_timestamp))
            if source_timestamp > target_timestamp =>
        {
            target.last_compacted_row_count = Some(source_rows);
        }
        _ => {}
    }
}

pub(super) fn merge_latest_epoch_timestamp(
    target_timestamp: &mut Option<String>,
    target_epoch: &mut Option<u64>,
    source_timestamp: Option<&str>,
    source_epoch: Option<u64>,
) {
    match (target_timestamp.as_deref(), source_timestamp) {
        (None, Some(source_timestamp)) => {
            *target_timestamp = Some(source_timestamp.to_string());
            *target_epoch = source_epoch;
        }
        (Some(current_timestamp), Some(source_timestamp))
            if source_timestamp > current_timestamp =>
        {
            *target_timestamp = Some(source_timestamp.to_string());
            *target_epoch = source_epoch;
        }
        _ => {
            if target_epoch.is_none() {
                *target_epoch = source_epoch;
            }
        }
    }
}

pub(super) fn merge_latest_reason_timestamp(
    target_timestamp: &mut Option<String>,
    target_reason: &mut Option<String>,
    source_timestamp: Option<&str>,
    source_reason: Option<&str>,
) {
    match (target_timestamp.as_deref(), source_timestamp) {
        (None, Some(source_timestamp)) => {
            *target_timestamp = Some(source_timestamp.to_string());
            *target_reason = source_reason.map(str::to_string);
        }
        (Some(current_timestamp), Some(source_timestamp))
            if source_timestamp > current_timestamp =>
        {
            *target_timestamp = Some(source_timestamp.to_string());
            *target_reason = source_reason.map(str::to_string);
        }
        _ => {
            if target_reason.is_none() {
                *target_reason = source_reason.map(str::to_string);
            }
        }
    }
}
