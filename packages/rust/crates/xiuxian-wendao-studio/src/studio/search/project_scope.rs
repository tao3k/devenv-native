use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::studio::pathing;
use crate::studio::types::UiProjectConfig;

#[derive(Debug, Clone, Default)]
pub(crate) struct SearchProjectMetadata {
    pub(crate) project_name: Option<String>,
    pub(crate) root_label: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ConfiguredProjectScope {
    pub(crate) scope_path: PathBuf,
    pub(crate) normalized_scope: String,
    pub(crate) project_name: String,
    pub(crate) root_label: Option<String>,
}

pub(super) fn normalize_path(path: &Path) -> String {
    normalize_pathbuf(path).to_string_lossy().replace('\\', "/")
}

pub(super) fn normalize_config_path(value: &str) -> Option<String> {
    pathing::normalize_project_dir_root(value)
}

pub(crate) fn configured_project_scan_roots(
    config_root: &Path,
    projects: &[UiProjectConfig],
) -> Vec<PathBuf> {
    configured_project_scopes(config_root, projects)
        .into_iter()
        .map(|scope| scope.scope_path)
        .collect()
}

pub(crate) fn configured_project_scopes(
    config_root: &Path,
    projects: &[UiProjectConfig],
) -> Vec<ConfiguredProjectScope> {
    let mut roots = Vec::new();
    let mut seen = HashSet::new();

    for project in projects {
        for configured_path in &project.dirs {
            let Some(normalized_configured_path) = normalize_config_path(configured_path.as_str())
            else {
                continue;
            };
            let Some(scope_path) = resolve_project_scope_path(
                config_root,
                project.root.as_str(),
                normalized_configured_path.as_str(),
            ) else {
                continue;
            };
            if !scope_path.exists() {
                continue;
            }
            let normalized_scope = normalize_path(scope_path.as_path());
            if seen.insert(normalized_scope.clone()) {
                roots.push(ConfiguredProjectScope {
                    scope_path,
                    normalized_scope,
                    project_name: project.name.clone(),
                    root_label: configured_root_label(
                        normalized_configured_path.as_str(),
                        project.name.as_str(),
                    ),
                });
            }
        }
    }

    roots
}

pub(crate) fn resolve_project_root_path(
    config_root: &Path,
    configured_root: &str,
) -> Option<PathBuf> {
    pathing::resolve_path_like(config_root, configured_root)
}

pub(super) fn resolve_project_scope_path(
    config_root: &Path,
    configured_root: &str,
    configured_path: &str,
) -> Option<PathBuf> {
    let project_base = resolve_project_root_path(config_root, configured_root)?;
    pathing::resolve_path_like(project_base.as_path(), configured_path)
}

pub(crate) fn index_path_for_entry(project_root: &Path, path: &Path) -> String {
    path.strip_prefix(project_root)
        .map_or_else(|_| normalize_path(path), normalize_path)
}

pub(crate) fn project_metadata_for_path(
    project_root: &Path,
    config_root: &Path,
    projects: &[UiProjectConfig],
    hit_path: &str,
) -> SearchProjectMetadata {
    let absolute_hit = if Path::new(hit_path).is_absolute() {
        PathBuf::from(hit_path)
    } else {
        project_root.join(hit_path)
    };
    let mut best_path_match: Option<(usize, SearchProjectMetadata)> = None;
    let mut best_root_match: Option<(usize, SearchProjectMetadata)> = None;

    for project in projects {
        let Some(project_root_path) = resolve_project_root_path(config_root, project.root.as_str())
        else {
            continue;
        };

        if !path_within_scope(absolute_hit.as_path(), project_root_path.as_path()) {
            continue;
        }

        update_best_match(
            &mut best_root_match,
            path_specificity(normalize_path(project_root_path.as_path()).as_str()),
            SearchProjectMetadata {
                project_name: Some(project.name.clone()),
                root_label: None,
            },
        );

        for configured_path in &project.dirs {
            let Some(normalized_path) = normalize_config_path(configured_path.as_str()) else {
                continue;
            };
            let Some(candidate_scope) = resolve_project_scope_path(
                config_root,
                project.root.as_str(),
                normalized_path.as_str(),
            ) else {
                continue;
            };
            if !path_within_scope(absolute_hit.as_path(), candidate_scope.as_path()) {
                continue;
            }

            update_best_match(
                &mut best_path_match,
                path_specificity(normalize_path(candidate_scope.as_path()).as_str()),
                SearchProjectMetadata {
                    project_name: Some(project.name.clone()),
                    root_label: configured_root_label(
                        normalized_path.as_str(),
                        project.name.as_str(),
                    ),
                },
            );
        }
    }

    best_path_match
        .map(|(_, metadata)| metadata)
        .or_else(|| best_root_match.map(|(_, metadata)| metadata))
        .unwrap_or_default()
}

fn configured_root_label(configured_path: &str, project_name: &str) -> Option<String> {
    if configured_path == "." {
        return Some(project_name.to_string());
    }

    Path::new(configured_path)
        .file_name()
        .map(|segment| segment.to_string_lossy().into_owned())
        .or_else(|| Some(project_name.to_string()))
}

fn path_within_scope(path: &Path, scope: &Path) -> bool {
    let normalized_path = normalize_pathbuf(path);
    let normalized_scope = normalize_pathbuf(scope);
    normalized_path == normalized_scope || normalized_path.strip_prefix(&normalized_scope).is_ok()
}

fn normalize_pathbuf(path: &Path) -> PathBuf {
    path.components()
        .fold(PathBuf::new(), |mut acc, component| {
            match component {
                std::path::Component::CurDir => {}
                other => acc.push(other.as_os_str()),
            }
            acc
        })
}

fn path_specificity(path: &str) -> usize {
    if path == "." {
        0
    } else {
        path.split('/').count()
    }
}

fn update_best_match(
    slot: &mut Option<(usize, SearchProjectMetadata)>,
    specificity: usize,
    metadata: SearchProjectMetadata,
) {
    match slot {
        Some((current_specificity, _)) if *current_specificity >= specificity => {}
        _ => *slot = Some((specificity, metadata)),
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/gateway/studio/search/project_scope.rs"]
mod tests;
