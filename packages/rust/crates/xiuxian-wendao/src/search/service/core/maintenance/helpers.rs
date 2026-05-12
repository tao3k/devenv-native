//! Compatibility path boundary: this module preserves an established Wendao owner path while the API surface is being narrowed.
use crate::repo_index::{RepoIndexEntryStatus, RepoIndexPhase};
use crate::search::service::core::types::RepoRuntimeState;
use crate::search::service::helpers::repo_corpus_active_epoch;
use crate::search::{SearchCorpusKind, SearchRepoCorpusRecord};

pub(super) const PREWARM_ROW_LIMIT: usize = 32;
pub(super) const COMPACTION_STARVATION_GUARD_ENQUEUE_LAG: u64 = 3;
pub(super) const LOCAL_MAINTENANCE_SHUTDOWN_MESSAGE: &str =
    "search-plane local maintenance runtime was stopped before completing task";
pub(crate) const REPO_MAINTENANCE_SHUTDOWN_MESSAGE: &str =
    "repo maintenance runtime was stopped before completing task";

pub(super) fn repo_active_epoch(
    corpus: SearchCorpusKind,
    repo_records: &[SearchRepoCorpusRecord],
) -> Option<u64> {
    let mut publication_epochs = repo_records
        .iter()
        .filter_map(|record| {
            record
                .publication
                .as_ref()
                .map(crate::search::SearchRepoPublicationRecord::active_epoch_value)
        })
        .collect::<Vec<_>>();
    if publication_epochs.is_empty() {
        return None;
    }
    publication_epochs.sort_unstable();
    Some(repo_corpus_active_epoch(corpus, &publication_epochs))
}

pub(super) fn repo_runtime_status_for_record(
    record: &SearchRepoCorpusRecord,
) -> Option<RepoIndexEntryStatus> {
    record
        .runtime
        .as_ref()
        .map(|runtime| RepoRuntimeState::from_record(runtime).as_status(record.repo_id.as_str()))
        .or_else(|| {
            record
                .publication
                .as_ref()
                .map(|publication| RepoIndexEntryStatus {
                    repo_id: record.repo_id.clone(),
                    phase: RepoIndexPhase::Idle,
                    queue_position: None,
                    last_error: None,
                    last_revision: publication.source_revision.clone(),
                    updated_at: Some(publication.published_at.clone()),
                    attempt_count: 0,
                })
        })
}
