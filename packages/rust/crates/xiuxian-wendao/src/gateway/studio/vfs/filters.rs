use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub type VfsError = crate::gateway::studio::StudioApiError;

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
