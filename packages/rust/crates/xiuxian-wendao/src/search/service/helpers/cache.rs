//! Compatibility path boundary: this module preserves an established Wendao owner path while the API surface is being narrowed.
use crate::repo_index::{RepoIndexEntryStatus, RepoIndexPhase};
use crate::search::service::SearchPlaneService;
use crate::search::service::core::RepoRuntimeState;
use crate::search::{SearchCorpusKind, SearchPlanePhase, SearchRepoPublicationRecord};

pub(crate) fn repo_corpus_fingerprint_part(
    repo: &RepoIndexEntryStatus,
    publication: &SearchRepoPublicationRecord,
) -> String {
    format!(
        "{}:{}:{}:{}:{}:{}:{}",
        repo.repo_id,
        publication.source_revision.as_deref().unwrap_or_default(),
        repo_phase_cache_fragment(repo.phase),
        repo.last_revision.as_deref().unwrap_or_default(),
        publication.table_version_id,
        publication.row_count,
        publication.fragment_count
    )
}

pub(crate) fn repo_corpus_active_epoch(
    corpus: SearchCorpusKind,
    publication_epochs: &[u64],
) -> u64 {
    let mut sorted_epochs = publication_epochs.to_vec();
    sorted_epochs.sort_unstable();
    sorted_epochs.dedup();
    stable_epoch_token(
        format!(
            "{corpus}:active:{}",
            sorted_epochs
                .iter()
                .map(u64::to_string)
                .collect::<Vec<_>>()
                .join("|")
        )
        .as_str(),
    )
}

pub(crate) fn repo_corpus_staging_epoch(
    corpus: SearchCorpusKind,
    repo_statuses: &[RepoIndexEntryStatus],
    active_epoch: Option<u64>,
) -> Option<u64> {
    let mut active_parts = repo_statuses
        .iter()
        .filter(|repo| {
            matches!(
                repo.phase,
                RepoIndexPhase::Queued
                    | RepoIndexPhase::Checking
                    | RepoIndexPhase::Syncing
                    | RepoIndexPhase::Indexing
            )
        })
        .map(|repo| {
            format!(
                "{}:{}:{}:{}",
                repo.repo_id,
                repo_phase_cache_fragment(repo.phase),
                repo.last_revision.as_deref().unwrap_or_default(),
                repo.updated_at.as_deref().unwrap_or_default()
            )
        })
        .collect::<Vec<_>>();
    if active_parts.is_empty() {
        return None;
    }
    active_parts.sort_unstable();
    Some(stable_epoch_token(
        format!(
            "{corpus}:staging:{}:{}",
            active_epoch.unwrap_or_default(),
            active_parts.join("|")
        )
        .as_str(),
    ))
}

pub(crate) fn stable_epoch_token(payload: &str) -> u64 {
    let hash = blake3::hash(payload.as_bytes());
    let mut bytes = [0_u8; 8];
    bytes.copy_from_slice(&hash.as_bytes()[..8]);
    u64::from_be_bytes(bytes)
}

impl SearchPlaneService {
    pub(crate) fn corpus_cache_version(&self, corpus: SearchCorpusKind) -> String {
        let status = self.coordinator().status_for(corpus);
        if let Some(epoch) = status.active_epoch {
            return format!("{corpus}:schema:{}:epoch:{epoch}", corpus.schema_version());
        }
        format!(
            "{corpus}:schema:{}:phase:{}",
            corpus.schema_version(),
            search_phase_cache_fragment(status.phase)
        )
    }
}

pub(crate) fn repo_corpus_cache_version(
    corpus: SearchCorpusKind,
    repo_id: &str,
    status: Option<&RepoRuntimeState>,
) -> String {
    let Some(status) = status else {
        return format!(
            "{corpus}:schema:{}:repo:{}:phase:missing",
            corpus.schema_version(),
            normalize_cache_fragment(repo_id)
        );
    };
    format!(
        "{corpus}:schema:{}:repo:{}:phase:{}:revision:{}:updated:{}",
        corpus.schema_version(),
        normalize_cache_fragment(repo_id),
        repo_phase_cache_fragment(status.phase),
        normalize_cache_fragment(status.last_revision.as_deref().unwrap_or_default()),
        normalize_cache_fragment(status.updated_at.as_deref().unwrap_or_default())
    )
}

pub(crate) fn repo_publication_cache_version(
    status: Option<&RepoRuntimeState>,
    publication: &SearchRepoPublicationRecord,
) -> String {
    let base = publication.cache_version();
    let Some(status) = status else {
        return base;
    };
    let published_revision =
        normalize_cache_fragment(publication.source_revision.as_deref().unwrap_or_default());
    let current_revision =
        normalize_cache_fragment(status.last_revision.as_deref().unwrap_or_default());
    if status.phase == RepoIndexPhase::Ready
        && (current_revision.is_empty() || current_revision == published_revision)
    {
        return base;
    }
    format!(
        "{base}:phase:{}:current-revision:{current_revision}:published-revision:{published_revision}",
        repo_phase_cache_fragment(status.phase)
    )
}

pub(crate) fn repo_phase_cache_fragment(phase: RepoIndexPhase) -> &'static str {
    match phase {
        RepoIndexPhase::Idle => "idle",
        RepoIndexPhase::Queued => "queued",
        RepoIndexPhase::Checking => "checking",
        RepoIndexPhase::Syncing => "syncing",
        RepoIndexPhase::Indexing => "indexing",
        RepoIndexPhase::Ready => "ready",
        RepoIndexPhase::Unsupported => "unsupported",
        RepoIndexPhase::Failed => "failed",
    }
}

pub(crate) fn search_phase_cache_fragment(phase: SearchPlanePhase) -> &'static str {
    match phase {
        SearchPlanePhase::Idle => "idle",
        SearchPlanePhase::Indexing => "indexing",
        SearchPlanePhase::Ready => "ready",
        SearchPlanePhase::Degraded => "degraded",
        SearchPlanePhase::Failed => "failed",
    }
}

pub(crate) fn normalize_cache_fragment(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}
