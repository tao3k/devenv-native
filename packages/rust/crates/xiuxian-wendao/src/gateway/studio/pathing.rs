use crate::analyzers::{RegisteredRepository, resolve_registered_repository_source};
use crate::gateway::studio::{StudioState, configured_repositories};
use crate::gateway::studio::types::UiProjectConfig;
use std::env;
use std::path::{Path, PathBuf};
use xiuxian_git_repo::SyncMode;

pub fn resolve_path_like(base: &Path, input: &str) -> Option<PathBuf> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }

    let expanded = expand_home_path_like(trimmed)?;
    let path = expanded.as_path();
    let joined = if path.is_absolute() {
        path.to_path_buf()
    } else {
        base.join(path)
    };
    normalize_path_buf_like(joined.as_path())
}

pub fn normalize_project_dir_root(dir: &str) -> Option<String> {
    normalize_path_like(dir)
}

pub fn normalize_path_like(raw: &str) -> Option<String> {
    let mut normalized = raw.trim().replace('\\', "/");
    if normalized.is_empty() {
        return None;
    }

    while normalized.contains("//") {
        normalized = normalized.replace("//", "/");
    }

    while normalized.len() > 1
        && normalized.ends_with('/')
        && !is_windows_drive_root(normalized.as_str())
    {
        normalized.pop();
    }

    if normalized.is_empty() {
        None
    } else {
        Some(normalized)
    }
}

pub fn studio_display_path(state: &StudioState, internal_path: &str) -> String {
    let fallback = normalize_path_like(internal_path)
        .unwrap_or_else(|| internal_path.trim().trim_start_matches('/').to_string());

    let projects = state.configured_projects();
    if projects.iter().any(|project| {
        fallback == project.name || fallback.starts_with(format!("{}/", project.name).as_str())
    }) {
        return fallback;
    }
    let repositories = configured_repositories(state);
    if repositories.iter().any(|repository| {
        fallback == repository.id || fallback.starts_with(format!("{}/", repository.id).as_str())
    }) {
        return fallback;
    }

    for project in &projects {
        if let Some(scoped_path) = project_scoped_display_path(state, project, fallback.as_str()) {
            return scoped_path;
        }
    }

    for repository in &repositories {
        if let Some(scoped_path) =
            repository_scoped_display_path(state, repository, fallback.as_str())
        {
            return scoped_path;
        }
    }

    fallback
}

pub(crate) fn studio_project_name(state: &StudioState, path: &str) -> Option<String> {
    let normalized = normalize_path_like(path)?;
    if let Some(project) = state.configured_projects().into_iter().find(|project| {
        normalized == project.name || normalized.starts_with(format!("{}/", project.name).as_str())
    }) {
        return Some(project.name);
    }

    configured_repositories(state)
        .into_iter()
        .find(|repository| {
            normalized == repository.id
                || normalized.starts_with(format!("{}/", repository.id).as_str())
        })
        .map(|repository| repository.id)
}

fn project_scoped_display_path(
    state: &StudioState,
    project: &UiProjectConfig,
    normalized_path: &str,
) -> Option<String> {
    let project_root = resolve_path_like(state.config_root.as_path(), project.root.as_str())?;
    let project_root_relative = project_root
        .strip_prefix(state.project_root.as_path())
        .ok()
        .and_then(|relative| {
            normalize_path_like(relative.to_string_lossy().as_ref())
                .filter(|value| !value.is_empty() && value != ".")
        });
    let relative_path = if Path::new(normalized_path).is_absolute() {
        let stripped = Path::new(normalized_path).strip_prefix(project_root).ok()?;
        normalize_path_like(stripped.to_string_lossy().as_ref())?
    } else if let Some(relative_root) = project_root_relative.as_deref() {
        if normalized_path == relative_root {
            normalized_path.to_string()
        } else if normalized_path.starts_with(format!("{relative_root}/").as_str()) {
            let prefix = format!("{relative_root}/");
            normalized_path.strip_prefix(prefix.as_str()).map_or_else(
                || normalized_path.to_string(),
                std::string::ToString::to_string,
            )
        } else {
            return None;
        }
    } else {
        normalized_path.to_string()
    };

    if !project_allows_relative_path(project, relative_path.as_str()) {
        return None;
    }

    Some(format!("{}/{}", project.name, relative_path))
}

fn repository_scoped_display_path(
    state: &StudioState,
    repository: &RegisteredRepository,
    normalized_path: &str,
) -> Option<String> {
    let source = resolve_registered_repository_source(
        repository,
        state.config_root.as_path(),
        SyncMode::Status,
    )
    .ok()?;
    let checkout_root = source.checkout_root;
    let checkout_root_relative = checkout_root
        .strip_prefix(state.project_root.as_path())
        .ok()
        .and_then(|relative| {
            normalize_path_like(relative.to_string_lossy().as_ref())
                .filter(|value| !value.is_empty() && value != ".")
        });
    let relative_path = if Path::new(normalized_path).is_absolute() {
        let stripped = Path::new(normalized_path)
            .strip_prefix(checkout_root.as_path())
            .ok()?;
        normalize_path_like(stripped.to_string_lossy().as_ref())?
    } else if let Some(relative_root) = checkout_root_relative.as_deref() {
        if normalized_path == relative_root {
            normalized_path.to_string()
        } else if normalized_path.starts_with(format!("{relative_root}/").as_str()) {
            let prefix = format!("{relative_root}/");
            normalized_path.strip_prefix(prefix.as_str()).map_or_else(
                || normalized_path.to_string(),
                std::string::ToString::to_string,
            )
        } else {
            return None;
        }
    } else {
        return None;
    };

    Some(format!("{}/{}", repository.id, relative_path))
}

fn project_allows_relative_path(project: &UiProjectConfig, relative_path: &str) -> bool {
    if project.dirs.is_empty() {
        return true;
    }

    project
        .dirs
        .iter()
        .filter_map(|dir| normalize_project_dir_root(dir))
        .any(|dir| {
            dir == "."
                || relative_path == dir
                || relative_path.starts_with(format!("{dir}/").as_str())
        })
}

fn is_windows_drive_root(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 3 && bytes[1] == b':' && bytes[2] == b'/'
}

fn expand_home_path_like(input: &str) -> Option<PathBuf> {
    if input == "~" {
        return home_dir();
    }

    if let Some(relative) = input
        .strip_prefix("~/")
        .or_else(|| input.strip_prefix("~\\"))
    {
        return home_dir().map(|path| path.join(relative));
    }

    Some(PathBuf::from(input))
}

fn normalize_path_buf_like(path: &Path) -> Option<PathBuf> {
    use std::path::Component;

    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            other => normalized.push(other.as_os_str()),
        }
    }

    if normalized.as_os_str().is_empty() {
        None
    } else {
        Some(normalized)
    }
}

fn home_dir() -> Option<PathBuf> {
    env::var_os("HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("USERPROFILE").map(PathBuf::from))
        .or_else(|| {
            let drive = env::var_os("HOMEDRIVE")?;
            let path = env::var_os("HOMEPATH")?;
            let mut combined = PathBuf::from(drive);
            combined.push(path);
            Some(combined)
        })
}

#[cfg(test)]
#[path = "../../../tests/unit/gateway/studio/pathing.rs"]
mod tests;
