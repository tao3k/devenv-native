//! Owns the Studio studio router repository surface.

use crate::studio::pathing;
use std::collections::HashSet;
use xiuxian_wendao::analyzers::{
    RegisteredRepository, RepoIntelligenceError, RepositoryPluginConfig, RepositoryRef,
    RepositoryRefreshPolicy,
};

use super::state::StudioState;

/// Returns the configured repository by ID.
///
/// # Errors
///
/// Returns an error when no configured repository matches `repo_id`.
pub(crate) fn configured_repository(
    studio: &StudioState,
    repo_id: &str,
) -> Result<RegisteredRepository, RepoIntelligenceError> {
    let repositories = configured_repositories(studio);
    if let Some(resolved_id) =
        resolve_registered_repository_id(repositories.as_slice(), repo_id).as_deref()
    {
        return repositories
            .into_iter()
            .find(|repository| repository.id.eq_ignore_ascii_case(resolved_id))
            .ok_or_else(|| RepoIntelligenceError::UnknownRepository {
                repo_id: resolved_id.to_string(),
            });
    }

    Err(RepoIntelligenceError::UnknownRepository {
        repo_id: repo_id.to_string(),
    })
}

/// Returns all configured repositories.
#[must_use]
pub fn configured_repositories(studio: &StudioState) -> Vec<RegisteredRepository> {
    studio
        .configured_repo_projects()
        .into_iter()
        .filter_map(|project| {
            if project.plugins.is_empty() {
                return None;
            }
            let path = project
                .root
                .as_deref()
                .and_then(|root| pathing::resolve_path_like(studio.config_root.as_path(), root));
            let url = project.url.map(|value| value.trim().to_string());
            if path.is_none() && url.is_none() {
                return None;
            }
            Some(RegisteredRepository {
                id: project.id,
                path,
                url,
                git_ref: project.git_ref.map(RepositoryRef::Branch),
                refresh: parse_refresh_policy(project.refresh.as_deref()),
                plugins: project
                    .plugins
                    .into_iter()
                    .map(RepositoryPluginConfig::Id)
                    .collect(),
            })
        })
        .collect()
}

#[must_use]
pub(crate) fn resolve_registered_repository_id(
    repositories: &[RegisteredRepository],
    repo_seed: &str,
) -> Option<String> {
    let trimmed = repo_seed.trim();
    if trimmed.is_empty() {
        return None;
    }

    if let Some(repository) = repositories
        .iter()
        .find(|repository| repository.id == trimmed)
    {
        return Some(repository.id.clone());
    }

    let mut case_insensitive = repositories
        .iter()
        .filter(|repository| repository.id.eq_ignore_ascii_case(trimmed));
    if let Some(repository) = case_insensitive.next() {
        if case_insensitive.next().is_none() {
            return Some(repository.id.clone());
        }
        return None;
    }

    let normalized_seed = normalize_repository_search_seed(trimmed);
    if normalized_seed.is_empty() {
        return None;
    }

    let mut matches = repositories.iter().filter(|repository| {
        registered_repository_search_seeds(repository).contains(normalized_seed.as_str())
    });
    let first = matches.next()?;
    if matches.next().is_some() {
        return None;
    }

    Some(first.id.clone())
}

pub(crate) fn registered_repository_search_seeds(
    repository: &RegisteredRepository,
) -> HashSet<String> {
    let mut seeds = HashSet::new();
    push_repository_search_seed(&mut seeds, repository.id.as_str());

    if let Some(path) = repository.path.as_deref()
        && let Some(file_name) = path.file_name().and_then(|value| value.to_str())
    {
        push_repository_search_seed(&mut seeds, file_name);
    }

    if let Some(url) = repository.url.as_deref() {
        push_repository_search_seed(&mut seeds, url);
        if let Some(last_segment) = repository_url_last_segment(url) {
            push_repository_search_seed(&mut seeds, last_segment);
        }
    }

    seeds
}

fn parse_refresh_policy(refresh: Option<&str>) -> RepositoryRefreshPolicy {
    match refresh
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("fetch")
    {
        "manual" => RepositoryRefreshPolicy::Manual,
        _ => RepositoryRefreshPolicy::Fetch,
    }
}

fn normalize_repository_search_seed(value: &str) -> String {
    let mut normalized = value.trim().to_ascii_lowercase();
    if let Some(stripped) = normalized.strip_suffix(".jl") {
        normalized = stripped.to_string();
    }

    let mut collapsed = String::with_capacity(normalized.len());
    let mut in_whitespace = true;
    for character in normalized.chars() {
        let mapped = if matches!(character, '_' | '.' | '/' | '-') {
            ' '
        } else {
            character
        };
        if mapped.is_ascii_whitespace() {
            if !in_whitespace {
                collapsed.push(' ');
            }
            in_whitespace = true;
        } else {
            collapsed.push(mapped);
            in_whitespace = false;
        }
    }

    collapsed.trim().to_string()
}

fn push_repository_search_seed(seeds: &mut HashSet<String>, value: &str) {
    let normalized = normalize_repository_search_seed(value);
    if !normalized.is_empty() {
        seeds.insert(normalized);
    }
}

fn repository_url_last_segment(url: &str) -> Option<&str> {
    let trimmed = url.trim().trim_end_matches('/');
    let last_segment = trimmed.rsplit('/').next()?;
    Some(last_segment.strip_suffix(".git").unwrap_or(last_segment))
}

#[cfg(test)]
#[path = "../../../tests/unit/gateway/studio/router/repository.rs"]
mod tests;
