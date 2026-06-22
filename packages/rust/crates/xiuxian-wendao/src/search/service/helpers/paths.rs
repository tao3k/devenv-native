//! Compatibility path boundary: this module preserves an established Wendao owner path while the API surface is being narrowed.
use std::path::{Path, PathBuf};

use xiuxian_config_core::ProjectDirs;

use crate::search::SearchManifestKeyspace;

pub(crate) fn default_storage_root(project_root: &Path) -> PathBuf {
    ProjectDirs::data_home()
        .join("wendao")
        .join("search_plane")
        .join(project_hash(project_root))
}

pub(crate) fn manifest_keyspace_for_project(project_root: &Path) -> SearchManifestKeyspace {
    SearchManifestKeyspace::new(format!(
        "xiuxian:wendao:search_plane:{}",
        project_hash(project_root)
    ))
}

pub(crate) fn project_hash(project_root: &Path) -> String {
    blake3::hash(project_root.to_string_lossy().as_bytes())
        .to_hex()
        .to_string()
}
