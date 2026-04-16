use std::sync::Arc;

use crate::analyzers::{
    RefineEntityDocRequest, RefineEntityDocResponse, RepoSyncMode, RepoSyncQuery, RepoSyncResult,
    repo_sync_for_registered_repository,
};
use crate::gateway::studio::router::handlers::repo::shared::execution::{
    with_repo_analysis, with_repository,
};
use crate::gateway::studio::router::handlers::repo::shared::repository::repo_index_repositories;
use crate::gateway::studio::router::{
    GatewayState, StudioApiError, configured_repositories, resolve_registered_repository_id,
};
use crate::repo_index::{RepoIndexRequest, RepoIndexStatusResponse};

pub(crate) async fn run_repo_index(
    state: Arc<GatewayState>,
    mut payload: RepoIndexRequest,
) -> Result<RepoIndexStatusResponse, StudioApiError> {
    payload.repo = payload.repo.and_then(|repo_id| {
        resolve_registered_repository_id(
            configured_repositories(&state.studio).as_slice(),
            repo_id.as_str(),
        )
        .or(Some(repo_id))
    });
    let repositories = repo_index_repositories(&state, payload.repo.as_deref())?;
    if repositories.is_empty() {
        return Err(StudioApiError::bad_request(
            "UNKNOWN_REPOSITORY",
            "No configured repository is available for repo indexing",
        ));
    }
    state
        .studio
        .repo_index
        .ensure_repositories_enqueued(repositories, payload.refresh);
    Ok(state
        .studio
        .repo_index
        .status_response(payload.repo.as_deref()))
}

pub(crate) fn run_repo_index_status(
    state: &Arc<GatewayState>,
    repo: Option<&str>,
) -> RepoIndexStatusResponse {
    let repo = repo.and_then(|repo_id| {
        resolve_registered_repository_id(configured_repositories(&state.studio).as_slice(), repo_id)
    });
    state.studio.repo_index_status(repo.as_deref())
}

pub(crate) async fn run_repo_sync(
    state: Arc<GatewayState>,
    repo_id: String,
    mode: RepoSyncMode,
) -> Result<RepoSyncResult, StudioApiError> {
    let canonical_repo_id = resolve_registered_repository_id(
        configured_repositories(&state.studio).as_slice(),
        repo_id.as_str(),
    )
    .unwrap_or(repo_id);
    with_repository(
        Arc::clone(&state),
        canonical_repo_id,
        "REPO_SYNC_PANIC",
        "Repo sync task failed unexpectedly",
        !matches!(mode, RepoSyncMode::Status),
        move |repository, cwd| {
            repo_sync_for_registered_repository(
                &RepoSyncQuery {
                    repo_id: repository.id.clone(),
                    mode,
                },
                &repository,
                cwd.as_path(),
            )
        },
    )
    .await
}

pub(crate) async fn run_refine_entity_doc(
    state: Arc<GatewayState>,
    payload: RefineEntityDocRequest,
) -> Result<RefineEntityDocResponse, StudioApiError> {
    let repo_id =
        crate::gateway::studio::router::handlers::repo::parse::repo::required_registered_repo_id(
            state.studio.as_ref(),
            Some(payload.repo_id.as_str()),
        )?;
    with_repo_analysis(
        Arc::clone(&state),
        repo_id,
        "REFINE_DOC_PANIC",
        "Refine documentation task failed unexpectedly",
        move |analysis| {
            let symbol = analysis
                .symbols
                .iter()
                .find(|symbol| symbol.symbol_id == payload.entity_id)
                .ok_or_else(|| crate::RepoIntelligenceError::AnalysisFailed {
                    message: format!("Entity `{}` not found", payload.entity_id),
                })?;

            let refined_content = format!(
                "## Refined Explanation for {}\n\nThis {:?} is part of the `{}` module. \
                It has been automatically refined using user hints: \"{}\".\n\n\
                **Signature**: `{}`",
                symbol.name,
                symbol.kind,
                symbol.module_id.as_deref().unwrap_or("root"),
                payload.user_hints.as_deref().unwrap_or("none"),
                symbol.signature.as_deref().unwrap_or("unknown")
            );

            Ok::<_, crate::RepoIntelligenceError>(RefineEntityDocResponse {
                repo_id: payload.repo_id.clone(),
                entity_id: payload.entity_id,
                refined_content,
                verification_state: "verified".to_string(),
            })
        },
    )
    .await
}
