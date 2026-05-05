use chrono::Utc;
use tokio::sync::OwnedSemaphorePermit;
use xiuxian_config_core::lookup_positive_parsed;
use xiuxian_db_store::VectorStoreError;

use crate::search::service::core::types::SearchPlaneService;

const DEFAULT_REPO_SEARCH_READ_CONCURRENCY_FALLBACK: usize = 2;
const MIN_REPO_SEARCH_READ_CONCURRENCY: usize = 1;
const MAX_REPO_SEARCH_READ_CONCURRENCY: usize = 4;
const REPO_SEARCH_READ_CONCURRENCY_ENV: &str = "XIUXIAN_WENDAO_REPO_SEARCH_READ_CONCURRENCY";

impl SearchPlaneService {
    #[doc(hidden)]
    pub async fn acquire_repo_search_read_permit(
        &self,
    ) -> Result<OwnedSemaphorePermit, VectorStoreError> {
        std::sync::Arc::clone(&self.repo_search_read_permits)
            .acquire_owned()
            .await
            .map_err(|_| {
                VectorStoreError::General("repo search read permits are unavailable".to_string())
            })
    }

    #[must_use]
    #[doc(hidden)]
    pub fn repo_search_parallelism(&self, repo_count: usize) -> usize {
        if repo_count == 0 {
            return 1;
        }

        self.repo_search_read_concurrency_limit
            .max(1)
            .min(repo_count)
    }

    /// Return the currently available repo-search read permits.
    #[must_use]
    pub fn available_repo_search_read_permits(&self) -> usize {
        self.repo_search_read_permits.available_permits()
    }

    pub(crate) fn record_repo_search_dispatch(
        &self,
        requested_repo_count: usize,
        searchable_repo_count: usize,
        parallelism: usize,
    ) {
        let mut runtime = self
            .repo_search_dispatch
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        runtime.captured_at = Some(Utc::now().to_rfc3339());
        runtime.requested_repo_count = u32::try_from(requested_repo_count).unwrap_or(u32::MAX);
        runtime.searchable_repo_count = u32::try_from(searchable_repo_count).unwrap_or(u32::MAX);
        runtime.parallelism = u32::try_from(parallelism).unwrap_or(u32::MAX);
        runtime.fanout_capped = searchable_repo_count > parallelism;
    }
}

pub(crate) fn repo_search_read_concurrency_limit() -> usize {
    repo_search_read_concurrency_limit_with_lookup(
        &|key| std::env::var(key).ok(),
        std::thread::available_parallelism()
            .ok()
            .map(std::num::NonZeroUsize::get),
    )
}

pub(crate) fn repo_search_read_concurrency_limit_with_lookup(
    lookup: &dyn Fn(&str) -> Option<String>,
    available_parallelism: Option<usize>,
) -> usize {
    lookup_positive_parsed::<usize>(REPO_SEARCH_READ_CONCURRENCY_ENV, lookup)
        .unwrap_or_else(|| default_repo_search_read_concurrency_limit(available_parallelism))
}

fn default_repo_search_read_concurrency_limit(available_parallelism: Option<usize>) -> usize {
    available_parallelism
        .unwrap_or(DEFAULT_REPO_SEARCH_READ_CONCURRENCY_FALLBACK)
        .div_ceil(5)
        .clamp(
            MIN_REPO_SEARCH_READ_CONCURRENCY,
            MAX_REPO_SEARCH_READ_CONCURRENCY,
        )
}
