//! Owns the Studio studio vfs filters surface.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Error type returned by Studio VFS filtering helpers.
pub type VfsError = crate::studio::StudioApiError;

/// File filter built from configured project VFS roots.
#[derive(Debug, Clone)]
pub struct ProjectFileFilter {
    pub root: PathBuf,
    pub allowed_subdirs: HashSet<PathBuf>,
}

impl ProjectFileFilter {
    pub fn matches(&self, path: &Path) -> bool {
        if !path.starts_with(&self.root) {
            return false;
        }
        if self.allowed_subdirs.is_empty() {
            return true;
        }
        self.allowed_subdirs
            .iter()
            .any(|subdir| path.starts_with(subdir))
    }
}
