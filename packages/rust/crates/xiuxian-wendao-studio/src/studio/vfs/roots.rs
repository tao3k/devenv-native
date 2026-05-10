use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::studio::pathing::resolve_path_like;
use crate::studio::types::VfsEntry;
use crate::studio::{StudioState, configured_repositories};
use xiuxian_git_repo::SyncMode;
use xiuxian_wendao::analyzers::RegisteredRepository;
use xiuxian_wendao::analyzers::resolve_registered_repository_source;

use super::filters::ProjectFileFilter;
use super::metadata::unix_timestamp_secs;

pub(crate) struct VfsRoot {
    pub request_root: String,
    pub full_path: PathBuf,
    pub project_name: Option<String>,
    pub root_label: Option<String>,
    pub filter_prefix: String,
    pub file_filters: Vec<ProjectFileFilter>,
}

pub(crate) fn list_root_entries(state: &StudioState) -> Vec<VfsEntry> {
    let mut entries = Vec::new();

    for root in resolve_all_vfs_roots(state) {
        let metadata = fs::metadata(root.full_path.as_path()).ok();
        let modified = metadata.as_ref().map_or(0, unix_timestamp_secs);
        let project_dirs = root.file_filters.first().map(|filter| {
            filter
                .allowed_subdirs
                .iter()
                .map(|path| path.to_string_lossy().to_string())
                .collect::<Vec<_>>()
        });
        entries.push(VfsEntry {
            path: root.request_root.clone().into(),
            name: root
                .root_label
                .clone()
                .or_else(|| root.project_name.clone())
                .unwrap_or_else(|| root.request_root.clone()),
            is_dir: metadata.as_ref().is_none_or(fs::Metadata::is_dir),
            size: metadata.as_ref().map_or(0, fs::Metadata::len),
            modified,
            content_type: None,
            project_name: root.project_name.clone(),
            root_label: root.root_label.clone(),
            project_root: Some(root.full_path.to_string_lossy().to_string()),
            project_dirs,
        });
    }

    entries.sort_by(|left, right| left.path.cmp(&right.path));
    entries
}

pub(super) fn resolve_all_vfs_roots(state: &StudioState) -> Vec<VfsRoot> {
    state
        .configured_projects()
        .iter()
        .filter_map(|project| resolve_project_vfs_root(state, project))
        .chain(
            configured_repositories(state)
                .into_iter()
                .filter_map(|repository| resolve_repo_vfs_root(state, &repository)),
        )
        .collect()
}

fn resolve_project_vfs_root(
    state: &StudioState,
    project: &crate::studio::types::UiProjectConfig,
) -> Option<VfsRoot> {
    let project_root = resolve_path_like(&state.config_root, project.root.as_str())?;
    let file_filters = compile_project_filters(&project_root, &project.dirs);
    Some(VfsRoot {
        request_root: project.name.clone(),
        full_path: project_root,
        project_name: Some(project.name.clone()),
        root_label: None,
        filter_prefix: String::new(),
        file_filters,
    })
}

fn resolve_repo_vfs_root(
    state: &StudioState,
    repository: &RegisteredRepository,
) -> Option<VfsRoot> {
    let source = resolve_registered_repository_source(
        repository,
        state.config_root.as_path(),
        SyncMode::Status,
    )
    .ok()?;

    if !source.checkout_root.is_dir() {
        return None;
    }

    let checkout_root = source.checkout_root;
    Some(VfsRoot {
        request_root: repository.id.clone(),
        full_path: checkout_root.clone(),
        project_name: Some(repository.id.clone()),
        root_label: None,
        filter_prefix: String::new(),
        file_filters: vec![ProjectFileFilter {
            root: checkout_root,
            allowed_subdirs: HashSet::new(),
        }],
    })
}

fn compile_project_filters(root: &Path, dirs: &[String]) -> Vec<ProjectFileFilter> {
    let mut allowed_subdirs = HashSet::new();
    for dir in dirs {
        allowed_subdirs.insert(root.join(dir));
    }
    vec![ProjectFileFilter {
        root: root.to_path_buf(),
        allowed_subdirs,
    }]
}
