//! Shared path resolution for `orgize` command execution.

use std::path::{Path, PathBuf};

use xiuxian_db_store::state::{ProjectCacheRootConfig, project_cache_root_from_config};

use crate::ClientContext;

pub(super) fn resolve_sdd_paths(paths: &[PathBuf], context: &ClientContext) -> Vec<PathBuf> {
    if paths.is_empty() {
        return vec![project_cache_root(context).join("agent").join("sdd")];
    }
    resolve_paths(paths, context)
}

pub(super) fn resolve_paths(paths: &[PathBuf], context: &ClientContext) -> Vec<PathBuf> {
    let paths = if paths.is_empty() {
        vec![PathBuf::from(".")]
    } else {
        paths.to_vec()
    };
    paths
        .into_iter()
        .map(|path| {
            if path.is_absolute() {
                path
            } else {
                context.root().join(path)
            }
        })
        .collect()
}

pub(super) fn display_path(path: &Path, context: &ClientContext) -> String {
    path.strip_prefix(context.root()).map_or_else(
        |_| path.display().to_string(),
        |path| path.display().to_string(),
    )
}

fn project_cache_root(context: &ClientContext) -> PathBuf {
    project_cache_root_from_config(ProjectCacheRootConfig {
        project_root: Some(context.root().to_path_buf()),
        cache_home: None,
        project_namespace: None,
    })
}
