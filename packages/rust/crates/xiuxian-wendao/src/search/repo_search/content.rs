//! Compatibility path boundary: this module preserves an established Wendao owner path while the API surface is being narrowed.
//! Coordinates repository search across source discovery, AST extraction, and batch rendering.

use std::collections::HashSet;
use std::path::Path;

use walkdir::{DirEntry, WalkDir};
use xiuxian_git_repo::SyncMode;
use xiuxian_wendao_runtime::transport::RepoSearchFlightRequest;

use crate::analyzers::{RegisteredRepository, resolve_registered_repository_source};
use crate::parsers::search::repo_code_query::parse_repo_code_search_query;
use crate::search::contracts::SearchHit;
use crate::search::repo_content_chunk::RepoContentChunkCandidate;
use crate::search::{RepoContentChunkSearchFilters, SearchPlaneService};

/// Search repository content rows using a raw code-search query.
///
/// # Errors
///
/// Returns a string error when the repository content query fails.
/// Primitive boundary: this public API keeps raw Wendao identifier carriers for existing transport and query contracts.
pub async fn search_repo_content_hits_for_query(
    search_plane: &SearchPlaneService,
    repo_id: &str,
    raw_query: &str,
    limit: usize,
) -> Result<Vec<SearchHit>, String> {
    let parsed = parse_repo_code_search_query(raw_query);
    let Some(search_term) = parsed.search_term() else {
        return Ok(Vec::new());
    };
    if !parsed.kind_filters.is_empty() && !parsed.kind_filters.contains("file") {
        return Ok(Vec::new());
    }

    search_repo_content_hits(
        search_plane,
        &RepoSearchFlightRequest {
            repo_id: repo_id.to_string(),
            query_text: search_term.to_string(),
            limit,
            language_filters: parsed.language_filters.clone(),
            path_prefixes: std::collections::HashSet::new(),
            title_filters: std::collections::HashSet::new(),
            tag_filters: std::collections::HashSet::new(),
            filename_filters: std::collections::HashSet::new(),
        },
    )
    .await
}

pub(crate) async fn search_repo_content_hits(
    search_plane: &SearchPlaneService,
    request: &RepoSearchFlightRequest,
) -> Result<Vec<SearchHit>, String> {
    let repo_id = request.repo_id.trim();
    if repo_id.is_empty() {
        return Err("repo-search request repo_id must not be blank".to_string());
    }

    search_plane
        .search_repo_content_chunks_with_filters(
            repo_id,
            request.query_text.as_str(),
            &request.language_filters,
            &RepoContentChunkSearchFilters {
                path_prefixes: request.path_prefixes.clone(),
                filename_filters: request.filename_filters.clone(),
                title_filters: request.title_filters.clone(),
                tag_filters: request.tag_filters.clone(),
            },
            request.limit,
        )
        .await
        .map_err(|error| format!("repo-search content query failed for repo `{repo_id}`: {error}"))
}

pub(crate) async fn search_repo_content_hits_with_repository(
    search_plane: &SearchPlaneService,
    repository: Option<&RegisteredRepository>,
    request: &RepoSearchFlightRequest,
) -> Result<Vec<SearchHit>, String> {
    let published_hits = search_repo_content_hits(search_plane, request).await?;
    if !published_hits.is_empty()
        || request.query_text.trim().is_empty()
        || !request.title_filters.is_empty()
        || !request.tag_filters.is_empty()
    {
        return Ok(published_hits);
    }

    let Some(repository) = repository else {
        return Ok(published_hits);
    };
    if !repository_supports_content_checkout_analysis(repository) {
        return Ok(published_hits);
    }

    search_repo_checkout_content_hits(search_plane, repository, request).await
}

async fn search_repo_checkout_content_hits(
    search_plane: &SearchPlaneService,
    repository: &RegisteredRepository,
    request: &RepoSearchFlightRequest,
) -> Result<Vec<SearchHit>, String> {
    let repository = repository.clone();
    let project_root = search_plane.project_root().to_path_buf();
    let request = request.clone();

    tokio::task::spawn_blocking(move || {
        search_repo_checkout_content_hits_blocking(project_root.as_path(), &repository, &request)
    })
    .await
    .map_err(|error| format!("repo-search checkout fallback task failed: {error}"))?
}

fn repository_supports_content_checkout_analysis(repository: &RegisteredRepository) -> bool {
    repository
        .plugins
        .iter()
        .any(|plugin| plugin.id().eq_ignore_ascii_case("ast-grep"))
}

fn search_repo_checkout_content_hits_blocking(
    project_root: &Path,
    repository: &RegisteredRepository,
    request: &RepoSearchFlightRequest,
) -> Result<Vec<SearchHit>, String> {
    if request.limit == 0 {
        return Ok(Vec::new());
    }

    let materialized =
        resolve_registered_repository_source(repository, project_root, SyncMode::Ensure).map_err(
            |error| format!("failed to resolve repository `{}`: {error}", repository.id),
        )?;
    let checkout_root = materialized.checkout_root;
    let normalized_query = request.query_text.trim().to_ascii_lowercase();
    let normalized_language_filters = normalize_filters(&request.language_filters);
    let normalized_path_prefixes = normalize_filters(&request.path_prefixes);
    let normalized_filename_filters = normalize_filters(&request.filename_filters);
    let mut hits = Vec::new();

    for entry in WalkDir::new(checkout_root.as_path())
        .into_iter()
        .filter_entry(should_descend_into_source_entry)
    {
        let Ok(entry) = entry else { continue };
        if !entry.file_type().is_file() {
            continue;
        }

        let relative_path = normalize_repo_relative_path(checkout_root.as_path(), entry.path());
        let normalized_relative_path = relative_path.to_ascii_lowercase();
        if !normalized_path_prefixes.is_empty()
            && !normalized_path_prefixes
                .iter()
                .any(|prefix| normalized_relative_path.starts_with(prefix.as_str()))
        {
            continue;
        }

        let Some(file_name) = entry.path().file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        let normalized_file_name = file_name.to_ascii_lowercase();
        if !normalized_filename_filters.is_empty()
            && !normalized_filename_filters.contains(normalized_file_name.as_str())
        {
            continue;
        }

        let language = infer_repo_source_language(relative_path.as_str());
        if !normalized_language_filters.is_empty() {
            let Some(language) = language.as_deref() else {
                continue;
            };
            if !normalized_language_filters.contains(language) {
                continue;
            }
        }

        let Ok(content) = std::fs::read_to_string(entry.path()) else {
            continue;
        };
        for (line_index, line) in content.lines().enumerate() {
            if !line
                .to_ascii_lowercase()
                .contains(normalized_query.as_str())
            {
                continue;
            }

            let exact_match = line.trim().eq_ignore_ascii_case(request.query_text.trim());
            hits.push(
                RepoContentChunkCandidate {
                    path: relative_path.clone(),
                    language: language.clone(),
                    line_number: line_index + 1,
                    line_text: line.to_string(),
                    score: source_search_score(line, request.query_text.as_str(), exact_match),
                    exact_match,
                }
                .into_search_hit(repository.id.as_str()),
            );
            if hits.len() >= request.limit {
                return Ok(hits);
            }
        }
    }

    Ok(hits)
}

fn normalize_filters(filters: &HashSet<String>) -> HashSet<String> {
    filters
        .iter()
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty())
        .collect()
}

fn infer_repo_source_language(path: &str) -> Option<String> {
    source_language_id_from_path(Path::new(path)).map(str::to_string)
}

fn source_language_id_from_path(path: &Path) -> Option<&'static str> {
    match path.extension().and_then(std::ffi::OsStr::to_str) {
        Some(ext) if ext.eq_ignore_ascii_case("rs") => Some("rust"),
        Some(ext) if ext.eq_ignore_ascii_case("py") => Some("python"),
        Some(ext) if ext.eq_ignore_ascii_case("ts") || ext.eq_ignore_ascii_case("tsx") => {
            Some("typescript")
        }
        Some(ext) if ext.eq_ignore_ascii_case("js") || ext.eq_ignore_ascii_case("jsx") => {
            Some("javascript")
        }
        Some(ext) if ext.eq_ignore_ascii_case("jl") => Some("julia"),
        Some(ext) if ext.eq_ignore_ascii_case("mo") => Some("modelica"),
        _ => None,
    }
}

fn source_search_score(line: &str, query: &str, exact_match: bool) -> f64 {
    if exact_match {
        return 1.0;
    }

    let normalized_line = line.trim().to_ascii_lowercase();
    let normalized_query = query.trim().to_ascii_lowercase();
    if normalized_line.starts_with(normalized_query.as_str()) {
        return 0.95;
    }

    0.82
}

fn normalize_repo_relative_path(checkout_root: &Path, path: &Path) -> String {
    path.strip_prefix(checkout_root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn should_descend_into_source_entry(entry: &DirEntry) -> bool {
    if entry.depth() == 0 || !entry.file_type().is_dir() {
        return true;
    }

    let Some(name) = entry.file_name().to_str() else {
        return false;
    };
    !matches!(
        name,
        ".git" | ".jj" | ".svn" | ".hg" | ".direnv" | "target" | "node_modules"
    )
}
