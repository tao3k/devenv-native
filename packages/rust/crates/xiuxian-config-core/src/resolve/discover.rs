//! Candidate discovery for global and orphan config files.

use std::path::{Path, PathBuf};

pub(super) fn global_candidates(config_home: Option<&Path>) -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    if let Some(config_home) = config_home {
        candidates.push(
            config_home
                .join("xiuxian-artisan-workshop")
                .join("xiuxian.toml"),
        );
    }
    candidates
}

pub(super) fn orphan_candidates(config_home: Option<&Path>, orphan_file: &str) -> Vec<PathBuf> {
    if orphan_file.trim().is_empty() {
        return Vec::new();
    }

    let mut candidates = Vec::new();
    if let Some(config_home) = config_home {
        candidates.push(
            config_home
                .join("xiuxian-artisan-workshop")
                .join(orphan_file),
        );
    }
    candidates
}

pub(super) fn existing_config_files(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    paths.into_iter().filter(|path| path.is_file()).collect()
}
