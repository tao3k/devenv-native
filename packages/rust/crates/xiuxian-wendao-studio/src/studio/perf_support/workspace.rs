use std::sync::Arc;
use std::time::Duration;

use anyhow::{Result, anyhow};
use tokio::time::{Instant, sleep};

use crate::studio::{GatewayState, configured_repositories, studio_effective_wendao_toml_path};
use xiuxian_wendao::analyzers::{
    analyze_registered_repository_with_registry, load_repo_intelligence_config,
};
use xiuxian_wendao::repo_index::RepoCodeDocument;
use xiuxian_wendao::repo_index::RepoIndexStatusResponse;

use crate::studio::perf_support::root::real_workspace_ready_timeout;

pub(crate) async fn warm_real_workspace_search_plane(state: &Arc<GatewayState>) -> Result<()> {
    let repositories = configured_repositories(&state.studio);
    if repositories.is_empty() {
        return Err(anyhow!(
            "real workspace fixture requires at least one configured repository"
        ));
    }

    state
        .studio
        .repo_index
        .ensure_repositories_enqueued(repositories.clone(), false);
    wait_for_repo_index_ready(state, repositories.len()).await
}

async fn wait_for_repo_index_ready(
    state: &Arc<GatewayState>,
    expected_repositories: usize,
) -> Result<()> {
    let timeout = real_workspace_ready_timeout();
    let start = Instant::now();
    loop {
        let status = state.studio.repo_index.status_response(None);
        if real_workspace_status_is_query_ready(&status, expected_repositories) {
            return Ok(());
        }

        if start.elapsed() >= timeout {
            return Err(anyhow!(
                "timed out waiting for repo index bootstrap after {:?} (total={}, ready={}, unsupported={}, failed={}, active={}, queued={}, checking={}, syncing={}, indexing={})",
                timeout,
                status.total,
                status.ready,
                status.unsupported,
                status.failed,
                status.active,
                status.queued,
                status.checking,
                status.syncing,
                status.indexing
            ));
        }

        sleep(Duration::from_secs(1)).await;
    }
}

pub(crate) fn real_workspace_status_is_query_ready(
    status: &RepoIndexStatusResponse,
    expected_repositories: usize,
) -> bool {
    status.total >= expected_repositories && status.ready > 0
}

pub(crate) async fn publish_code_search_snapshot(
    state: &Arc<GatewayState>,
    repo_id: &str,
) -> Result<()> {
    let config_path = studio_effective_wendao_toml_path(state.studio.config_root.as_path());
    let config = load_repo_intelligence_config(
        Some(config_path.as_path()),
        state.studio.config_root.as_path(),
    )?;
    let repository = config
        .repos
        .iter()
        .find(|repository| repository.id == repo_id)
        .ok_or_else(|| anyhow!("repository `{repo_id}` not found in perf config"))?;
    let analysis = analyze_registered_repository_with_registry(
        repository,
        state.studio.config_root.as_path(),
        &state.studio.plugin_registry,
    )?;

    state
        .studio
        .search_plane
        .publish_repo_entities_with_revision(
            repo_id,
            &analysis,
            &[
                RepoCodeDocument {
                    path: "src/GatewaySyncPkg.jl".to_string(),
                    language: Some("julia".to_string()),
                    contents: Arc::<str>::from(
                        "module GatewaySyncPkg\nexport solve\nsolve() = nothing\nend\n",
                    ),
                    size_bytes: 56,
                    modified_unix_ms: 0,
                },
                RepoCodeDocument {
                    path: "examples/solve_demo.jl".to_string(),
                    language: Some("julia".to_string()),
                    contents: Arc::<str>::from("using GatewaySyncPkg\nsolve()\n"),
                    size_bytes: 29,
                    modified_unix_ms: 0,
                },
            ],
            None,
        )
        .await?;
    state
        .studio
        .search_plane
        .publish_repo_content_chunks_with_revision(
            repo_id,
            &[RepoCodeDocument {
                path: "src/GatewaySyncPkg.jl".to_string(),
                language: Some("julia".to_string()),
                contents: Arc::<str>::from(
                    "module GatewaySyncPkg\nexport solve\nsolve() = nothing\nend\n",
                ),
                size_bytes: 56,
                modified_unix_ms: 0,
            }],
            None,
        )
        .await?;
    Ok(())
}
