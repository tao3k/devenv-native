//! Shared Org file discovery and rendering helpers.

use std::fs;
use std::path::{Path, PathBuf};

use super::OrgizeToolError;

pub(in crate::orgize_tool) fn collect_org_paths(
    paths: &[PathBuf],
) -> Result<Vec<PathBuf>, OrgizeToolError> {
    let mut files = Vec::new();
    for path in paths {
        collect_org_path(path, &mut files)?;
    }
    files.sort();
    files.dedup();
    Ok(files)
}

fn collect_org_path(path: &Path, files: &mut Vec<PathBuf>) -> Result<(), OrgizeToolError> {
    let metadata = fs::metadata(path).map_err(|source| OrgizeToolError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    if metadata.is_file() {
        if !is_org_file(path) {
            return Err(OrgizeToolError::NotOrgFile {
                path: path.to_path_buf(),
            });
        }
        files.push(path.to_path_buf());
        return Ok(());
    }
    if !metadata.is_dir() {
        return Err(OrgizeToolError::UnsupportedPath {
            path: path.to_path_buf(),
        });
    }

    let mut entries = fs::read_dir(path)
        .map_err(|source| OrgizeToolError::Io {
            path: path.to_path_buf(),
            source,
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|source| OrgizeToolError::Io {
            path: path.to_path_buf(),
            source,
        })?;
    entries.sort_by_key(std::fs::DirEntry::path);

    for entry in entries {
        let entry_path = entry.path();
        let entry_type = entry.file_type().map_err(|source| OrgizeToolError::Io {
            path: entry_path.clone(),
            source,
        })?;
        if entry_type.is_dir() {
            collect_org_path(&entry_path, files)?;
        } else if entry_type.is_file() && is_org_file(&entry_path) {
            files.push(entry_path);
        }
    }
    Ok(())
}

fn is_org_file(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("org"))
}

pub(in crate::orgize_tool) fn read_to_string(path: &Path) -> Result<String, OrgizeToolError> {
    fs::read_to_string(path).map_err(|source| OrgizeToolError::Io {
        path: path.to_path_buf(),
        source,
    })
}

pub(in crate::orgize_tool) fn join_projection_text(
    rendered: Vec<String>,
    empty_text: &str,
) -> String {
    let non_empty = rendered
        .into_iter()
        .filter(|text| text.trim() != empty_text.trim())
        .collect::<Vec<_>>();
    if non_empty.is_empty() {
        empty_text.to_string()
    } else {
        non_empty.join("\n")
    }
}
