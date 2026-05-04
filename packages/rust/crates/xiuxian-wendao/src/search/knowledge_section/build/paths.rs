use std::path::Path;

use crate::search::contracts::SearchProjectConfig;
use crate::search::contracts::{SearchProjectMetadata, resolve_project_root_path};
#[cfg(test)]
use crate::search::fingerprint_note_projects;

pub(super) fn studio_display_path(
    project_root: &Path,
    config_root: &Path,
    projects: &[SearchProjectConfig],
    metadata: &SearchProjectMetadata,
    path: &str,
) -> String {
    let normalized = path.replace('\\', "/");
    if projects.len() > 1
        && let Some(project_name) = metadata.project_name.as_deref()
    {
        let relative_to_project = projects
            .iter()
            .find(|project| project.name == project_name)
            .and_then(|project| resolve_project_root_path(config_root, project.root.as_str()))
            .and_then(|project_root_path| {
                let absolute_path = if Path::new(path).is_absolute() {
                    Path::new(path).to_path_buf()
                } else {
                    project_root.join(path)
                };
                absolute_path
                    .strip_prefix(project_root_path)
                    .ok()
                    .map(|relative| relative.to_string_lossy().replace('\\', "/"))
            })
            .filter(|relative| !relative.is_empty())
            .unwrap_or_else(|| normalized.clone());

        if !relative_to_project.starts_with(&format!("{project_name}/")) {
            return format!("{project_name}/{relative_to_project}");
        }
    }

    normalized
}

#[cfg(test)]
pub(super) fn fingerprint_projects(
    project_root: &Path,
    config_root: &Path,
    projects: &[SearchProjectConfig],
) -> String {
    fingerprint_note_projects(project_root, config_root, projects)
}
