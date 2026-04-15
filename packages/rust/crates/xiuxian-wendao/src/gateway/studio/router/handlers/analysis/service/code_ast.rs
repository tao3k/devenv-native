use std::fs;
use std::path::Path;
use std::sync::Arc;

use xiuxian_git_repo::SyncMode;

use crate::analyzers::resolve_registered_repository_source;
use crate::analyzers::service::analyze_registered_repository_target_file_with_registry;
use crate::gateway::studio::router::code_ast::{
    build_code_ast_analysis_response, build_generic_code_ast_analysis_response,
    resolve_code_ast_repository_and_path,
};
use crate::gateway::studio::router::{
    GatewayState, StudioApiError, configured_repositories, map_repo_intelligence_error,
};
use crate::gateway::studio::types::CodeAstAnalysisResponse;
use crate::search::repo_search::repository_generic_ast_lang_for_path;

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
        let materialized =
            resolve_registered_repository_source(&repository, cwd.as_path(), SyncMode::Ensure)
                .map_err(|error| {
                    StudioApiError::internal(
                        "REPOSITORY_SOURCE_RESOLUTION_FAILED",
                        "Failed to resolve repository source for code AST analysis",
                        Some(error.to_string()),
                    )
                })?;
        let source_path = materialized.checkout_root.join(&repo_path);
        let source_content = source_path
            .is_file()
            .then(|| fs::read_to_string(source_path).ok())
            .flatten();

        if let Some(lang) = repository_generic_ast_lang_for_path(&repository, Path::new(&repo_path))
            && let Some(source_content) = source_content.as_deref()
        {
            let mut response = build_generic_code_ast_analysis_response(
                repo_id.clone(),
                repo_path.clone(),
                line_hint,
                source_content,
                lang,
            );
            response.path.clone_from(&request_path);
            return Ok(response);
        }

        let analysis = analyze_registered_repository_target_file_with_registry(
            &repository,
            cwd.as_path(),
            &plugin_registry,
            repo_path.as_str(),
        )
        .map_err(map_repo_intelligence_error)?;
        let mut response = build_code_ast_analysis_response(
            repo_id,
            repo_path,
            line_hint,
            source_content.as_deref(),
            &analysis,
        );
        response.path = request_path;
        Ok(response)
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
