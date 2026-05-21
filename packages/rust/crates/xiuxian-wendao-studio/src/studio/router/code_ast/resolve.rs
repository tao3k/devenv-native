//! Owns the Studio router code ast resolve surface.

use std::path::Path;

use xiuxian_wendao::analyzers::{RepoSymbolKind, RepositoryAnalysisOutput};

/// Resolve the repository context and normalized repository-relative path for code-AST queries.
///
/// # Errors
///
/// Returns [`crate::studio::router::StudioApiError`] when the
/// repository cannot be resolved from the explicit repo id or path prefix.
pub(crate) fn resolve_code_ast_repository_and_path<'a>(
    repositories: &'a [xiuxian_wendao::analyzers::RegisteredRepository],
    repo_id: Option<&str>,
    path: &str,
) -> Result<
    (&'a xiuxian_wendao::analyzers::RegisteredRepository, String),
    crate::studio::router::StudioApiError,
> {
    if let Some(id) = repo_id {
        let repo = repositories.iter().find(|r| r.id == id).ok_or_else(|| {
            crate::studio::router::StudioApiError::bad_request(
                "UNKNOWN_REPO",
                format!("Repository `{id}` not found"),
            )
        })?;
        if let Some(conflicting_repo) = repositories
            .iter()
            .find(|candidate| candidate.id != repo.id && path_has_repo_prefix(path, &candidate.id))
        {
            return Err(crate::studio::router::StudioApiError::bad_request(
                "REPO_PATH_MISMATCH",
                format!(
                    "Path `{path}` is scoped to repository `{}` and cannot be analyzed under `{}`",
                    conflicting_repo.id, repo.id
                ),
            ));
        }
        return Ok((repo, strip_repo_prefix(path, &repo.id)));
    }

    // Heuristic: try to find repo id in path prefix
    for repo in repositories {
        let prefix = format!("{}/", repo.id);
        if path.starts_with(&prefix) {
            return Ok((repo, path[prefix.len()..].to_string()));
        }
    }

    Err(crate::studio::router::StudioApiError::bad_request(
        "MISSING_REPO",
        "Repository context is required",
    ))
}

fn strip_repo_prefix(path: &str, repo_id: &str) -> String {
    if path == repo_id {
        return String::new();
    }
    let prefix = format!("{repo_id}/");
    if path.starts_with(&prefix) {
        return path[prefix.len()..].to_string();
    }
    path.to_string()
}

fn path_has_repo_prefix(path: &str, repo_id: &str) -> bool {
    path == repo_id || path.starts_with(&format!("{repo_id}/"))
}

pub(crate) fn repo_relative_path_matches(path: &str, target: &str) -> bool {
    path == target || path.ends_with(&format!("/{target}"))
}

pub(crate) fn path_has_extension(path: &str, extension: &str) -> bool {
    Path::new(path)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case(extension))
}

pub(crate) fn retrieval_semantic_type(
    symbol: &xiuxian_wendao::analyzers::SymbolRecord,
    same_file: bool,
) -> &'static str {
    if !same_file {
        return "externalSymbol";
    }

    if let Some(parser_kind) = symbol.attributes.get("parser_kind").map(String::as_str) {
        match parser_kind {
            "module" => return "module",
            "macro" => return "macro",
            "parameter" => return "parameter",
            "binding" => {
                if matches!(
                    symbol.attributes.get("binding_kind").map(String::as_str),
                    Some("const")
                ) {
                    return "constant";
                }
                return "binding";
            }
            "type" => return "type",
            "function" => return "function",
            _ => {}
        }
    }

    match symbol.kind {
        RepoSymbolKind::Function => "function",
        RepoSymbolKind::Type => "type",
        RepoSymbolKind::Constant => "constant",
        _ => "other",
    }
}

pub(crate) fn focus_symbol_for_blocks<'a>(
    line_hint: Option<usize>,
    analysis: &'a RepositoryAnalysisOutput,
    path: &str,
) -> Option<&'a xiuxian_wendao::analyzers::SymbolRecord> {
    if let Some(line) = line_hint {
        return analysis.symbols.iter().find(|symbol| {
            if !repo_relative_path_matches(symbol.path.as_str(), path) {
                return false;
            }
            match (symbol.line_start, symbol.line_end) {
                (Some(start), Some(end)) => start <= line && line <= end,
                (Some(start), None) => start == line,
                _ => false,
            }
        });
    }

    analysis
        .symbols
        .iter()
        .find(|symbol| repo_relative_path_matches(symbol.path.as_str(), path))
}
