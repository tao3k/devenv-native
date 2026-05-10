use crate::studio::router::{
    StudioApiError, StudioState, configured_repositories, resolve_registered_repository_id,
};

pub(crate) fn required_repo_id(repo: Option<&str>) -> Result<String, StudioApiError> {
    repo.map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .ok_or_else(|| StudioApiError::bad_request("MISSING_REPO", "`repo` is required"))
}

pub(crate) fn required_registered_repo_id(
    studio: &StudioState,
    repo: Option<&str>,
) -> Result<String, StudioApiError> {
    let repo_id = required_repo_id(repo)?;
    let repositories = configured_repositories(studio);
    Ok(
        resolve_registered_repository_id(repositories.as_slice(), repo_id.as_str())
            .unwrap_or(repo_id),
    )
}
