use std::fs;
use std::path::Path;
use std::sync::Arc;

use xiuxian_code_intelligence::CodeLanguageId;
use xiuxian_git_repo::MaterializedRepo;
use xiuxian_git_repo::SyncMode;
#[cfg(feature = "julia")]
use xiuxian_julia_core::fetch_modelica_ast_query_analysis_blocking_for_repository;
use xiuxian_wendao_core::repo_intelligence::RegisteredRepository;

use crate::studio::router::{
    GatewayState, StudioApiError, configured_repositories, map_repo_intelligence_error,
};
use crate::studio::router::{
    build_code_ast_analysis_response, build_generic_code_ast_analysis_response,
    resolve_code_ast_repository_and_path,
};
use crate::studio::types::CodeAstAnalysisResponse;
use xiuxian_wendao::analyzers::analyze_registered_repository_target_file_with_registry;
use xiuxian_wendao::analyzers::resolve_registered_repository_source;
use xiuxian_wendao::search::repo_search::repository_generic_ast_lang_for_path;

pub(crate) async fn load_code_ast_analysis_response(
    state: &GatewayState,
    path: &str,
    repo_id: &str,
    line_hint: Option<usize>,
) -> Result<CodeAstAnalysisResponse, StudioApiError> {
    let cwd = state.studio.project_root.clone();
    let repositories = configured_repositories(&state.studio);
    let (repository, repo_relative_path) =
        resolve_code_ast_repository_and_path(&repositories, Some(repo_id), path)?;
    let plugin_registry = Arc::clone(&state.studio.plugin_registry);

    let repo_id = repository.id.clone();
    let request_path = path.to_string();
    let repo_path = repo_relative_path;
    let repository = repository.clone();

    tokio::task::spawn_blocking(move || -> Result<CodeAstAnalysisResponse, StudioApiError> {
        load_code_ast_analysis_response_blocking(
            &repository,
            cwd.as_path(),
            &plugin_registry,
            repo_id.as_str(),
            request_path.as_str(),
            repo_path.as_str(),
            line_hint,
        )
    })
    .await
    .map_err(|error: tokio::task::JoinError| {
        StudioApiError::internal(
            "CODE_AST_PANIC",
            "Code AST analysis task failed unexpectedly",
            Some(error.to_string()),
        )
    })?
}

fn load_code_ast_analysis_response_blocking(
    repository: &RegisteredRepository,
    cwd: &Path,
    plugin_registry: &Arc<xiuxian_wendao::analyzers::PluginRegistry>,
    repo_id: &str,
    request_path: &str,
    repo_path: &str,
    line_hint: Option<usize>,
) -> Result<CodeAstAnalysisResponse, StudioApiError> {
    let materialized = resolve_code_ast_analysis_source(repository, cwd)?;
    let source_content = read_repo_source_content(materialized.checkout_root.as_path(), repo_path);

    if let Some(response) = maybe_build_generic_code_ast_response(
        repository,
        repo_id,
        request_path,
        repo_path,
        line_hint,
        source_content.as_deref(),
    ) {
        return Ok(response);
    }

    #[cfg(feature = "julia")]
    if let Some(response) = maybe_build_modelica_code_ast_response(
        repository,
        repo_id,
        request_path,
        repo_path,
        line_hint,
        source_content.as_deref(),
    )? {
        return Ok(response);
    }

    let analysis = analyze_registered_repository_target_file_with_registry(
        repository,
        cwd,
        plugin_registry,
        repo_path,
    )
    .map_err(map_repo_intelligence_error)?;
    Ok(finish_code_ast_analysis_response(
        repo_id,
        request_path,
        repo_path,
        line_hint,
        source_content.as_deref(),
        &analysis,
    ))
}

fn resolve_code_ast_analysis_source(
    repository: &RegisteredRepository,
    cwd: &Path,
) -> Result<MaterializedRepo, StudioApiError> {
    let status_source = resolve_registered_repository_source(repository, cwd, SyncMode::Status)
        .or_else(|_| resolve_registered_repository_source(repository, cwd, SyncMode::Ensure))
        .map_err(|error| map_repository_source_resolution_error(&error))?;
    if status_source.checkout_root.is_dir() {
        Ok(status_source)
    } else {
        resolve_registered_repository_source(repository, cwd, SyncMode::Ensure)
            .map_err(|error| map_repository_source_resolution_error(&error))
    }
}

fn read_repo_source_content(checkout_root: &Path, repo_path: &str) -> Option<String> {
    let source_path = checkout_root.join(repo_path);
    source_path
        .is_file()
        .then(|| fs::read_to_string(source_path).ok())
        .flatten()
}

fn maybe_build_generic_code_ast_response(
    repository: &RegisteredRepository,
    repo_id: &str,
    request_path: &str,
    repo_path: &str,
    line_hint: Option<usize>,
    source_content: Option<&str>,
) -> Option<CodeAstAnalysisResponse> {
    let lang = repository_generic_ast_lang_for_path(repository, Path::new(repo_path))?;
    let source_content = source_content?;
    let language_id = CodeLanguageId::from(lang.as_str());
    let mut response = build_generic_code_ast_analysis_response(
        repo_id.to_string(),
        repo_path.to_string(),
        line_hint,
        source_content,
        &language_id,
    );
    response.path = request_path.to_string().into();
    Some(response)
}

#[cfg(feature = "julia")]
fn maybe_build_modelica_code_ast_response(
    repository: &RegisteredRepository,
    repo_id: &str,
    request_path: &str,
    repo_path: &str,
    line_hint: Option<usize>,
    source_content: Option<&str>,
) -> Result<Option<CodeAstAnalysisResponse>, StudioApiError> {
    let source_content = match source_content {
        Some(source_content)
            if repository
                .plugins
                .iter()
                .any(|plugin| plugin.id() == "modelica")
                && Path::new(repo_path)
                    .extension()
                    .is_some_and(|extension| extension.eq_ignore_ascii_case("mo")) =>
        {
            source_content
        }
        _ => return Ok(None),
    };
    let analysis = fetch_modelica_ast_query_analysis_blocking_for_repository(
        repository,
        repo_path.into(),
        source_content,
    )
    .map_err(map_repo_intelligence_error)?;
    Ok(Some(finish_code_ast_analysis_response(
        repo_id,
        request_path,
        repo_path,
        line_hint,
        Some(source_content),
        &analysis,
    )))
}

fn finish_code_ast_analysis_response(
    repo_id: &str,
    request_path: &str,
    repo_path: &str,
    line_hint: Option<usize>,
    source_content: Option<&str>,
    analysis: &xiuxian_wendao::analyzers::RepositoryAnalysisOutput,
) -> CodeAstAnalysisResponse {
    let mut response = build_code_ast_analysis_response(
        repo_id.to_string(),
        repo_path.to_string(),
        line_hint,
        source_content,
        analysis,
    );
    response.path = request_path.to_string().into();
    response
}

fn map_repository_source_resolution_error(
    error: &xiuxian_wendao::analyzers::RepoIntelligenceError,
) -> StudioApiError {
    StudioApiError::internal(
        "REPOSITORY_SOURCE_RESOLUTION_FAILED",
        "Failed to resolve repository source for code AST analysis",
        Some(error.to_string()),
    )
}
