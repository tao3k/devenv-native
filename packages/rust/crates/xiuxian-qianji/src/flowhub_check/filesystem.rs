use std::path::{Path, PathBuf};

use globset::Glob;
use walkdir::WalkDir;

use crate::error::QianjiError;

pub(super) fn discover_immediate_child_directories(
    module_dir: &Path,
) -> Result<Vec<String>, QianjiError> {
    let mut child_dirs = Vec::new();
    for entry in std::fs::read_dir(module_dir).map_err(|error| {
        QianjiError::Topology(format!(
            "Failed to read Flowhub module directory `{}`: {error}",
            module_dir.display()
        ))
    })? {
        let entry = entry.map_err(|error| {
            QianjiError::Topology(format!(
                "Failed to read Flowhub module entry under `{}`: {error}",
                module_dir.display()
            ))
        })?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if name.starts_with('.') {
            continue;
        }
        child_dirs.push(name.to_string());
    }
    child_dirs.sort();
    Ok(child_dirs)
}

pub(super) fn discover_immediate_mermaid_files(
    module_dir: &Path,
) -> Result<Vec<PathBuf>, QianjiError> {
    let mut mermaid_files = Vec::new();
    for entry in std::fs::read_dir(module_dir).map_err(|error| {
        QianjiError::Topology(format!(
            "Failed to read Flowhub module directory `{}`: {error}",
            module_dir.display()
        ))
    })? {
        let entry = entry.map_err(|error| {
            QianjiError::Topology(format!(
                "Failed to read Flowhub module entry under `{}`: {error}",
                module_dir.display()
            ))
        })?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if path.extension().and_then(|extension| extension.to_str()) != Some("mmd") {
            continue;
        }
        mermaid_files.push(path);
    }
    mermaid_files.sort();
    Ok(mermaid_files)
}

pub(super) fn count_glob_matches(module_dir: &Path, pattern: &str) -> Result<usize, QianjiError> {
    let matcher = Glob::new(pattern)
        .map_err(|error| {
            QianjiError::Topology(format!(
                "invalid Flowhub module glob pattern `{pattern}`: {error}"
            ))
        })?
        .compile_matcher();

    let mut match_count = 0_usize;
    for entry in WalkDir::new(module_dir) {
        let entry = entry.map_err(|error| {
            QianjiError::Topology(format!(
                "Failed to walk Flowhub module directory `{}`: {error}",
                module_dir.display()
            ))
        })?;
        if entry.path() == module_dir {
            continue;
        }
        let relative = entry.path().strip_prefix(module_dir).map_err(|error| {
            QianjiError::Topology(format!(
                "Failed to relativize Flowhub module path `{}` against `{}`: {error}",
                entry.path().display(),
                module_dir.display()
            ))
        })?;
        let normalized = relative.to_string_lossy().replace('\\', "/");
        if matcher.is_match(normalized.as_str()) {
            match_count += 1;
        }
    }
    Ok(match_count)
}

pub(super) fn count_root_glob_matches(root: &Path, pattern: &str) -> Result<usize, QianjiError> {
    let matcher = Glob::new(pattern)
        .map_err(|error| {
            QianjiError::Topology(format!(
                "invalid Flowhub contract glob pattern `{pattern}`: {error}"
            ))
        })?
        .compile_matcher();

    let mut match_count = 0_usize;
    for entry in WalkDir::new(root) {
        let entry = entry.map_err(|error| {
            QianjiError::Topology(format!(
                "Failed to walk Flowhub root `{}`: {error}",
                root.display()
            ))
        })?;
        if entry.path() == root {
            continue;
        }
        let relative = entry.path().strip_prefix(root).map_err(|error| {
            QianjiError::Topology(format!(
                "Failed to relativize Flowhub root path `{}` against `{}`: {error}",
                entry.path().display(),
                root.display()
            ))
        })?;
        let normalized = relative.to_string_lossy().replace('\\', "/");
        if matcher.is_match(normalized.as_str()) {
            match_count += 1;
        }
    }

    Ok(match_count)
}

pub(super) fn last_module_segment(module_ref: &str) -> &str {
    module_ref.rsplit('/').next().unwrap_or(module_ref)
}

pub(super) fn is_glob_pattern(value: &str) -> bool {
    value
        .chars()
        .any(|character| matches!(character, '*' | '?' | '[' | ']'))
}
