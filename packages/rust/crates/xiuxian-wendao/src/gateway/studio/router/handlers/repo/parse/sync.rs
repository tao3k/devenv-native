use crate::analyzers::RepoSyncMode;
use crate::gateway::studio::router::StudioApiError;

pub(crate) fn parse_repo_sync_mode(mode: Option<&str>) -> Result<RepoSyncMode, StudioApiError> {
    match mode
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("ensure")
    {
        "ensure" => Ok(RepoSyncMode::Ensure),
        "refresh" => Ok(RepoSyncMode::Refresh),
        "status" => Ok(RepoSyncMode::Status),
        other => Err(StudioApiError::bad_request(
            "INVALID_MODE",
            format!("unsupported repo sync mode `{other}`"),
        )),
    }
}
