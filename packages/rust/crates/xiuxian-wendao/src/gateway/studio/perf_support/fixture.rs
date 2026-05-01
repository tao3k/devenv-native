use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;

use anyhow::{Result, anyhow};
use axum::Router;
#[cfg(feature = "julia")]
use xiuxian_wendao_julia::integration_support::{
    JuliaExampleServiceGuard, spawn_wendaosearch_julia_parser_summary_service_with_attempts,
};

#[cfg(feature = "julia")]
use crate::gateway::studio::perf_support::git::write_repo_config_with_julia_parser_summary_transport;
use crate::gateway::studio::perf_support::git::{create_local_git_repo, write_default_repo_config};
use crate::gateway::studio::perf_support::root::{
    DEFAULT_REAL_WORKSPACE_ROOT, GatewayPerfRoot, REAL_WORKSPACE_ROOT_ENV, create_perf_root,
    resolve_real_workspace_root,
};
use crate::gateway::studio::perf_support::state::gateway_state_for_project;
use crate::gateway::studio::perf_support::workspace::{
    publish_code_search_snapshot, warm_real_workspace_search_plane,
};
use crate::gateway::studio::studio_router;
use crate::repo_index::repo_index_policy_debug_snapshot;

#[cfg(feature = "julia")]
const PERF_JULIA_PARSER_SUMMARY_READY_ATTEMPTS: usize = 900;

/// Prepared Studio gateway fixture for performance tests.
pub struct GatewayPerfFixture {
    root: GatewayPerfRoot,
    state: Arc<crate::gateway::studio::GatewayState>,
    #[cfg(feature = "julia")]
    _julia_parser_summary_guard: Option<JuliaExampleServiceGuard>,
}

/// One controller-side concurrency snapshot captured alongside repo-index
/// performance audit samples.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GatewayRepoIndexControllerDebugSnapshot {
    /// Current controller target concurrency.
    pub target_concurrency: usize,
    /// Maximum controller ceiling derived from host parallelism.
    pub max_concurrency: usize,
    /// Number of successive stable completions at the current limit.
    pub success_streak: usize,
    /// Concurrency limit currently used by the baseline reference.
    pub reference_limit: usize,
    /// Number of consecutive I/O pressure observations at the current limit.
    pub io_pressure_streak: usize,
    /// Exponential moving average of sync control elapsed time in milliseconds.
    pub ema_elapsed_ms: Option<u64>,
    /// Rolling baseline used to detect sustained sync I/O pressure.
    pub baseline_elapsed_ms: Option<u64>,
    /// Most recent sync control elapsed time in milliseconds.
    pub last_elapsed_ms: Option<u64>,
    /// Ratio between current and previous efficiency, expressed as a percent.
    pub last_efficiency_ratio_pct: Option<u64>,
    /// Last controller adjustment reason recorded by the adaptive loop.
    pub last_adjustment: String,
    /// Effective analysis timeout used by the repo-index runtime.
    pub analysis_timeout_secs: u64,
    /// Effective sync timeout used by the repo-index runtime.
    pub sync_timeout_secs: u64,
    /// Effective retry budget for retryable sync failures.
    pub sync_retry_budget: usize,
}

impl GatewayPerfFixture {
    /// Return a fresh Studio router bound to the prepared gateway state.
    pub fn router(&self) -> Router {
        studio_router(Arc::clone(&self.state))
    }

    /// Return the shared gateway state backing this fixture.
    #[must_use]
    pub fn state(&self) -> Arc<crate::gateway::studio::GatewayState> {
        Arc::clone(&self.state)
    }

    /// Return one repo-index controller debug snapshot for performance probes.
    #[must_use]
    pub fn repo_index_controller_debug_snapshot(&self) -> GatewayRepoIndexControllerDebugSnapshot {
        let snapshot = self.state.studio.repo_index.controller_debug_snapshot();
        let policy = repo_index_policy_debug_snapshot();
        GatewayRepoIndexControllerDebugSnapshot {
            target_concurrency: snapshot.current_limit,
            max_concurrency: snapshot.max_limit,
            success_streak: snapshot.success_streak,
            reference_limit: snapshot.reference_limit,
            io_pressure_streak: snapshot.io_pressure_streak,
            ema_elapsed_ms: snapshot.ema_elapsed_ms,
            baseline_elapsed_ms: snapshot.baseline_elapsed_ms,
            last_elapsed_ms: snapshot.last_elapsed_ms,
            last_efficiency_ratio_pct: snapshot.last_efficiency_ratio_pct,
            last_adjustment: snapshot.last_adjustment.as_str().to_string(),
            analysis_timeout_secs: policy.analysis_timeout_secs,
            sync_timeout_secs: policy.sync_timeout_secs,
            sync_retry_budget: policy.sync_retry_budget,
        }
    }

    /// Return the temporary project root backing this fixture.
    #[must_use]
    pub fn root(&self) -> &Path {
        match &self.root {
            GatewayPerfRoot::Owned(path) | GatewayPerfRoot::External(path) => path.as_path(),
        }
    }

    /// Execute one direct repo-scoped search-plane query so status telemetry
    /// exposes a repo-specific scope bucket without paying for the full HTTP
    /// search handler chain.
    ///
    /// # Errors
    ///
    /// Returns an error if the repo-backed search-plane query fails or does
    /// not return any hits for the requested repo/query pair.
    pub async fn warm_repo_scope_query(&self, repo_id: &str, query: &str) -> Result<()> {
        let hits = self
            .state
            .studio
            .search_plane
            .search_repo_entities(repo_id, query, &HashSet::new(), &HashSet::new(), 5)
            .await
            .map_err(|error| anyhow!("failed to warm repo-scoped search telemetry: {error}"))?;
        if hits.is_empty() {
            return Err(anyhow!(
                "repo-scoped search warmup returned no hits for repo `{repo_id}` and query `{query}`"
            ));
        }
        Ok(())
    }
}

impl Drop for GatewayPerfFixture {
    fn drop(&mut self) {
        self.state.studio.stop_background_services();
        if let GatewayPerfRoot::Owned(path) = &self.root {
            let _ = std::fs::remove_dir_all(path);
        }
    }
}

/// Build a warm-cache gateway fixture with one Julia repository.
///
/// # Errors
///
/// Returns an error if the temporary project cannot be created, initialized as
/// a Git repository, analyzed, or published into the search plane.
pub async fn prepare_gateway_perf_fixture() -> Result<GatewayPerfFixture> {
    let root = create_perf_root()?;
    let repo_dir = create_local_git_repo(root.as_path(), "GatewaySyncPkg")?;
    write_default_repo_config(root.as_path(), repo_dir.as_path(), "gateway-sync")?;
    let state = gateway_state_for_project(root.as_path())?;
    publish_code_search_snapshot(&state, "gateway-sync").await?;
    Ok(GatewayPerfFixture {
        root: GatewayPerfRoot::Owned(root),
        state,
        #[cfg(feature = "julia")]
        _julia_parser_summary_guard: None,
    })
}

/// Build a warm-cache gateway fixture with one Julia repository and an active
/// parser-summary transport.
///
/// This helper keeps the spawned parser-summary service alive for the
/// fixture lifetime so repo-intelligence bootstrap and routed search queries
/// observe a stable Julia analysis surface.
///
/// # Errors
///
/// Returns an error if the temporary project cannot be created, the Julia
/// parser-summary service cannot be configured, initialized as a Git
/// repository, analyzed, or published into the search plane.
#[cfg(feature = "julia")]
pub async fn prepare_gateway_perf_fixture_with_julia_parser_summary_transport()
-> Result<GatewayPerfFixture> {
    let root = create_perf_root()?;
    let repo_dir = create_local_git_repo(root.as_path(), "GatewaySyncPkg")?;
    let (base_url, guard) = spawn_wendaosearch_julia_parser_summary_service_with_attempts(
        PERF_JULIA_PARSER_SUMMARY_READY_ATTEMPTS,
    )
    .await;
    write_repo_config_with_julia_parser_summary_transport(
        root.as_path(),
        repo_dir.as_path(),
        "gateway-sync",
        &base_url,
    )?;
    let state = gateway_state_for_project(root.as_path())?;
    publish_code_search_snapshot(&state, "gateway-sync").await?;
    Ok(GatewayPerfFixture {
        root: GatewayPerfRoot::Owned(root),
        state,
        _julia_parser_summary_guard: Some(guard),
    })
}

/// Build a warm-cache gateway fixture backed by a real multi-repository
/// workspace.
///
/// The workspace root is resolved from
/// `XIUXIAN_WENDAO_GATEWAY_PERF_WORKSPACE_ROOT` first, then falls back to
/// `.data/wendao-frontend` under the current project root when present.
///
/// # Errors
///
/// Returns an error when no real workspace root can be resolved, when the
/// target workspace cannot bootstrap gateway state, or when repo indexing does
/// not reach a query-ready state before the configured timeout.
pub async fn prepare_gateway_real_workspace_perf_fixture() -> Result<GatewayPerfFixture> {
    let root = resolve_real_workspace_root().ok_or_else(|| {
        anyhow!(
            "real gateway perf workspace root is not available; set {REAL_WORKSPACE_ROOT_ENV} or create {DEFAULT_REAL_WORKSPACE_ROOT}",
        )
    })?;
    let state = gateway_state_for_project(root.as_path())?;
    warm_real_workspace_search_plane(&state).await?;
    Ok(GatewayPerfFixture {
        root: GatewayPerfRoot::External(root),
        state,
        #[cfg(feature = "julia")]
        _julia_parser_summary_guard: None,
    })
}
