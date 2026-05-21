use std::sync::Arc;

use crate::studio::router::{
    GatewayState, StudioApiError, configured_repositories, configured_repository,
    map_repo_intelligence_error,
};
use xiuxian_wendao::analyzers::RegisteredRepository;

pub(super) fn resolve_repository(
    state: &Arc<GatewayState>,
    repo_id: &str,
) -> Result<RegisteredRepository, StudioApiError> {
    configured_repository(&state.studio, repo_id).map_err(map_repo_intelligence_error)
}

pub(crate) fn repo_index_repositories(
    state: &Arc<GatewayState>,
    repo: Option<&str>,
) -> Result<Vec<RegisteredRepository>, StudioApiError> {
    if let Some(repo_id) = repo {
        return Ok(vec![resolve_repository(state, repo_id)?]);
    }
    Ok(configured_repositories(&state.studio))
}

pub(super) fn repository_uses_managed_remote_source(repository: &RegisteredRepository) -> bool {
    repository.url.is_some()
}

#[cfg(test)]
#[path = "../../../../../../tests/unit/gateway/studio/router/handlers/repo/shared/repository.rs"]
mod tests;
