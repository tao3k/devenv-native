use std::time::Duration;

const MAX_SYNC_CONCURRENCY: usize = 16;
const MIN_ANALYSIS_TIMEOUT_SECS: u64 = 45;
const MAX_ANALYSIS_TIMEOUT_SECS: u64 = 180;
const ANALYSIS_TIMEOUT_BASE_SECS: u64 = 24;
const ANALYSIS_TIMEOUT_SECS_PER_CORE: u64 = 3;
const MIN_SYNC_TIMEOUT_SECS: u64 = 90;
const MAX_SYNC_TIMEOUT_SECS: u64 = 240;
const SYNC_TIMEOUT_BASE_SECS: u64 = 40;
const SYNC_TIMEOUT_SECS_PER_WORKER: u64 = 10;
const MAX_SYNC_REQUEUE_ATTEMPTS: usize = 3;

pub(crate) fn default_repo_index_sync_concurrency() -> usize {
    default_repo_index_sync_concurrency_for_parallelism(available_parallelism())
}

pub(crate) fn default_repo_index_sync_concurrency_for_parallelism(parallelism: usize) -> usize {
    let parallelism = parallelism.max(1);
    if parallelism == 1 {
        return 1;
    }

    parallelism
        .saturating_mul(2)
        .div_ceil(3)
        .clamp(2, MAX_SYNC_CONCURRENCY)
}

pub(crate) fn default_repo_index_sync_timeout() -> Duration {
    default_repo_index_sync_timeout_for_parallelism(available_parallelism())
}

pub(crate) fn default_repo_index_analysis_timeout() -> Duration {
    default_repo_index_analysis_timeout_for_parallelism(available_parallelism())
}

pub(crate) fn default_repo_index_sync_requeue_attempts() -> usize {
    default_repo_index_sync_requeue_attempts_for_parallelism(available_parallelism())
}

pub(crate) fn default_repo_index_sync_timeout_for_parallelism(parallelism: usize) -> Duration {
    let concurrency = u64::try_from(default_repo_index_sync_concurrency_for_parallelism(
        parallelism,
    ))
    .unwrap_or(u64::MAX);
    let timeout_secs = SYNC_TIMEOUT_BASE_SECS
        .saturating_add(concurrency.saturating_mul(SYNC_TIMEOUT_SECS_PER_WORKER))
        .clamp(MIN_SYNC_TIMEOUT_SECS, MAX_SYNC_TIMEOUT_SECS);
    Duration::from_secs(timeout_secs)
}

pub(crate) fn default_repo_index_analysis_timeout_for_parallelism(parallelism: usize) -> Duration {
    let parallelism = u64::try_from(parallelism.max(1)).unwrap_or(u64::MAX);
    let timeout_secs = ANALYSIS_TIMEOUT_BASE_SECS
        .saturating_add(parallelism.saturating_mul(ANALYSIS_TIMEOUT_SECS_PER_CORE))
        .clamp(MIN_ANALYSIS_TIMEOUT_SECS, MAX_ANALYSIS_TIMEOUT_SECS);
    Duration::from_secs(timeout_secs)
}

pub(crate) fn default_repo_index_sync_requeue_attempts_for_parallelism(
    parallelism: usize,
) -> usize {
    parallelism
        .max(1)
        .div_ceil(8)
        .clamp(1, MAX_SYNC_REQUEUE_ATTEMPTS)
}

fn available_parallelism() -> usize {
    std::thread::available_parallelism()
        .map_or(1, std::num::NonZeroUsize::get)
        .max(1)
}

#[cfg(test)]
#[path = "../../../../tests/unit/repo_index/state/task/machine.rs"]
mod tests;
