use std::sync::Arc;
use std::{ffi::OsStr, path::Path};

use crate::studio::router::{
    GatewayState, StudioApiError, configured_repository, map_repo_intelligence_error,
};
use xiuxian_wendao::analyzers::{RegisteredRepository, RepoOverviewResult};
use xiuxian_wendao::search::{
    RepoEntityOverviewSummary, RepoEntitySearchError, summarize_repo_entity_overview,
};

pub(crate) async fn run_repo_overview(
    state: Arc<GatewayState>,
    repo_id: String,
) -> Result<RepoOverviewResult, StudioApiError> {
    let repository = configured_repository(&state.studio, repo_id.as_str())
        .map_err(map_repo_intelligence_error)?;
    if !repository.has_repo_intelligence_plugins() {
        return Ok(build_search_only_repo_overview(&repository));
    }
    if let Some(summary) =
        summarize_repo_entity_overview(&state.studio.search_plane, repo_id.as_str())
            .await
            .map_err(|error| map_repo_entity_overview_error(&error))?
    {
        return Ok(build_repo_publication_overview(
            &repository,
            repo_id.as_str(),
            summary,
        ));
    }
    Err(StudioApiError::index_not_ready("repo_entity"))
}

fn build_repo_publication_overview(
    repository: &RegisteredRepository,
    repo_id: &str,
    summary: RepoEntityOverviewSummary,
) -> RepoOverviewResult {
    let display_name = summary
        .display_name
        .clone()
        .unwrap_or_else(|| repository_display_name(repository));
    RepoOverviewResult {
        repo_id: repo_id.to_string(),
        display_name,
        revision: summary.source_revision,
        module_count: summary.module_count,
        symbol_count: summary.symbol_count,
        example_count: summary.example_count,
        doc_count: summary.doc_count,
        hierarchical_uri: Some(format!("repo://{repo_id}")),
        hierarchy: Some(vec!["repo".to_string(), repo_id.to_string()]),
    }
}

fn build_search_only_repo_overview(repository: &RegisteredRepository) -> RepoOverviewResult {
    RepoOverviewResult {
        repo_id: repository.id.clone(),
        display_name: repository.id.clone(),
        revision: None,
        module_count: 0,
        symbol_count: 0,
        example_count: 0,
        doc_count: 0,
        hierarchical_uri: Some(format!("repo://{}", repository.id)),
        hierarchy: Some(vec!["repo".to_string(), repository.id.clone()]),
    }
}

fn repository_display_name(repository: &RegisteredRepository) -> String {
    repository
        .path
        .as_deref()
        .and_then(path_file_name)
        .or_else(|| repository.url.as_deref().and_then(repo_name_from_url))
        .filter(|display_name| !display_name.trim().is_empty())
        .unwrap_or_else(|| repository.id.clone())
}

fn path_file_name(path: &Path) -> Option<String> {
    path.file_name()
        .and_then(OsStr::to_str)
        .map(str::trim)
        .filter(|segment| !segment.is_empty())
        .map(ToOwned::to_owned)
}

fn repo_name_from_url(url: &str) -> Option<String> {
    url.trim()
        .trim_end_matches('/')
        .rsplit('/')
        .next()
        .map(|segment| segment.trim_end_matches(".git"))
        .filter(|segment| !segment.is_empty())
        .map(ToOwned::to_owned)
}

fn map_repo_entity_overview_error(error: &RepoEntitySearchError) -> StudioApiError {
    StudioApiError::internal(
        "REPO_OVERVIEW_PUBLICATION_ERROR",
        "Repo overview publication summary failed unexpectedly",
        Some(error.to_string()),
    )
}
