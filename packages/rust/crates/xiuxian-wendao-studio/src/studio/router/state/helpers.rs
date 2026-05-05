use std::collections::HashSet;
use std::path::Path;

use crate::studio::pathing;
use crate::studio::types::UiProjectConfig;

/// Returns the supported code kinds exposed by the Studio API.
#[must_use]
pub(crate) fn supported_code_kinds() -> Vec<String> {
    [
        "function",
        "method",
        "struct",
        "module",
        "class",
        "trait",
        "interface",
        "enum",
        "constant",
        "const",
        "macro",
        "type",
        "example",
        "doc",
        "reference",
    ]
    .into_iter()
    .map(ToOwned::to_owned)
    .collect()
}

pub(crate) fn graph_include_dirs(
    project_root: &Path,
    config_root: &Path,
    projects: &[UiProjectConfig],
) -> Vec<String> {
    let mut seen = HashSet::<String>::new();
    let mut include_dirs = Vec::new();

    for project in projects {
        let Some(project_base) = pathing::resolve_path_like(config_root, project.root.as_str())
        else {
            continue;
        };
        for dir_entry in &project.dirs {
            let Some(dir) = pathing::normalize_project_dir_root(dir_entry.as_str()) else {
                continue;
            };
            let Some(candidate) = pathing::resolve_path_like(project_base.as_path(), dir.as_str())
            else {
                continue;
            };
            let Ok(relative) = candidate.strip_prefix(project_root) else {
                continue;
            };
            let normalized = relative
                .to_string_lossy()
                .replace('\\', "/")
                .trim_end_matches('/')
                .to_string();
            let value = if normalized.is_empty() {
                ".".to_string()
            } else {
                normalized
            };
            if seen.insert(value.clone()) {
                include_dirs.push(value);
            }
        }
    }

    include_dirs
}
