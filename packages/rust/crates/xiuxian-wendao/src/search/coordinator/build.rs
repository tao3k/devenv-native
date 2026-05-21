//! Build lease state transitions for the search plane coordinator.

use crate::search::{
    SearchCorpusKind, SearchCorpusStatus, SearchPlaneCoordinator, SearchPlanePhase,
};

use super::state::timestamp_now;
use super::types::{BeginBuildDecision, SearchBuildLease, SearchCorpusRuntime};

impl SearchPlaneCoordinator {
    /// Attempt to start a new staging build for a corpus fingerprint.
    pub fn begin_build(
        &self,
        corpus: SearchCorpusKind,
        fingerprint: impl Into<String>,
        schema_version: u32,
    ) -> BeginBuildDecision {
        let _spawn_guard = self
            .spawn_lock
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let fingerprint = fingerprint.into();
        let now = timestamp_now();
        let mut state = self
            .state
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let runtime = state
            .entry(corpus)
            .or_insert_with(|| super::types::SearchCorpusRuntime::new(corpus));
        if let Some(decision) =
            current_build_decision(runtime, fingerprint.as_str(), schema_version)
        {
            return decision;
        }

        let epoch = start_staging_build(runtime, fingerprint.as_str(), schema_version, now);

        BeginBuildDecision::Started(SearchBuildLease {
            corpus,
            fingerprint,
            epoch,
            schema_version,
        })
    }

    /// Update build progress for a live lease.
    pub fn update_progress(&self, lease: &SearchBuildLease, progress: f32) -> bool {
        let mut state = self
            .state
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let Some(runtime) = state.get_mut(&lease.corpus) else {
            return false;
        };
        if !matches!(runtime.status.phase, SearchPlanePhase::Indexing)
            || runtime.status.staging_epoch != Some(lease.epoch)
            || runtime.status.fingerprint.as_deref() != Some(lease.fingerprint.as_str())
        {
            return false;
        }
        runtime.status.progress = Some(progress.clamp(0.0, 1.0));
        runtime.status.updated_at = Some(timestamp_now());
        true
    }

    /// Publish a completed staging epoch if the lease is still current.
    pub fn publish_ready(
        &self,
        lease: &SearchBuildLease,
        row_count: u64,
        fragment_count: u64,
    ) -> bool {
        let mut state = self
            .state
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let Some(runtime) = state.get_mut(&lease.corpus) else {
            return false;
        };
        if !lease_matches_staging_runtime(runtime, lease) {
            return false;
        }

        let now = timestamp_now();
        let publish_count = next_publish_count(runtime, lease.corpus);
        let compaction_pending = lease.corpus.supports_local_store_compaction()
            && self.maintenance_policy.should_compact(
                publish_count,
                runtime.last_compacted_row_count,
                row_count,
            );
        publish_ready_status(
            &mut runtime.status,
            lease,
            row_count,
            fragment_count,
            publish_count,
            compaction_pending,
            now,
        );
        true
    }

    /// Mark an in-flight build as failed if the lease is still current.
    pub fn fail_build(&self, lease: &SearchBuildLease, error: impl Into<String>) -> bool {
        let mut state = self
            .state
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let Some(runtime) = state.get_mut(&lease.corpus) else {
            return false;
        };
        if runtime.status.staging_epoch != Some(lease.epoch)
            || runtime.status.fingerprint.as_deref() != Some(lease.fingerprint.as_str())
        {
            return false;
        }

        let now = timestamp_now();
        runtime.status.phase = SearchPlanePhase::Failed;
        runtime.status.staging_epoch = None;
        runtime.status.progress = None;
        runtime.status.build_finished_at = Some(now.clone());
        runtime.status.updated_at = Some(now);
        runtime.status.last_error = Some(error.into());
        true
    }
}

fn current_build_decision(
    runtime: &SearchCorpusRuntime,
    fingerprint: &str,
    schema_version: u32,
) -> Option<BeginBuildDecision> {
    if !runtime_matches_requested_build(runtime, fingerprint, schema_version) {
        return None;
    }
    match runtime.status.phase {
        SearchPlanePhase::Ready if runtime.status.active_epoch.is_some() => {
            Some(BeginBuildDecision::AlreadyReady(runtime.status.clone()))
        }
        SearchPlanePhase::Indexing => {
            Some(BeginBuildDecision::AlreadyIndexing(runtime.status.clone()))
        }
        _ => None,
    }
}

fn runtime_matches_requested_build(
    runtime: &SearchCorpusRuntime,
    fingerprint: &str,
    schema_version: u32,
) -> bool {
    runtime.status.schema_version == schema_version
        && runtime.status.fingerprint.as_deref() == Some(fingerprint)
}

fn start_staging_build(
    runtime: &mut SearchCorpusRuntime,
    fingerprint: &str,
    schema_version: u32,
    now: String,
) -> u64 {
    let epoch = runtime.next_epoch;
    runtime.next_epoch = runtime.next_epoch.saturating_add(1);
    runtime.status.phase = SearchPlanePhase::Indexing;
    runtime.status.staging_epoch = Some(epoch);
    runtime.status.schema_version = schema_version;
    runtime.status.fingerprint = Some(fingerprint.to_string());
    runtime.status.progress = Some(0.0);
    runtime.status.build_started_at = Some(now.clone());
    runtime.status.build_finished_at = None;
    runtime.status.updated_at = Some(now);
    runtime.status.last_error = None;
    epoch
}

fn lease_matches_staging_runtime(runtime: &SearchCorpusRuntime, lease: &SearchBuildLease) -> bool {
    runtime.status.staging_epoch == Some(lease.epoch)
        && runtime.status.fingerprint.as_deref() == Some(lease.fingerprint.as_str())
}

fn next_publish_count(runtime: &SearchCorpusRuntime, corpus: SearchCorpusKind) -> u32 {
    if corpus.supports_local_store_compaction() {
        runtime
            .status
            .maintenance
            .publish_count_since_compaction
            .saturating_add(1)
    } else {
        0
    }
}

fn publish_ready_status(
    status: &mut SearchCorpusStatus,
    lease: &SearchBuildLease,
    row_count: u64,
    fragment_count: u64,
    publish_count: u32,
    compaction_pending: bool,
    now: String,
) {
    status.phase = SearchPlanePhase::Ready;
    status.active_epoch = Some(lease.epoch);
    status.staging_epoch = None;
    status.schema_version = lease.schema_version;
    status.progress = None;
    status.row_count = Some(row_count);
    status.fragment_count = Some(fragment_count);
    status.build_finished_at = Some(now.clone());
    status.updated_at = Some(now);
    status.last_error = None;
    status.maintenance.publish_count_since_compaction = publish_count;
    status.maintenance.compaction_pending = compaction_pending;
}
