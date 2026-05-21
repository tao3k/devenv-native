use std::fs::{self, DirEntry};
use std::path::{Path, PathBuf};

use globset::Glob;
use walkdir::WalkDir;

use crate::error::QianjiError;

pub(super) fn discover_immediate_child_directories(
    module_dir: &Path,
) -> Result<Vec<String>, QianjiError> {
    let mut child_dirs = fs::read_dir(module_dir)
        .map_err(|error| {
            QianjiError::Topology(format!(
                "Failed to read Flowhub module directory `{}`: {error}",
                module_dir.display()
            ))
        })?
        .map(|entry| child_directory_name(module_dir, entry))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    child_dirs.sort();
    Ok(child_dirs)
}

pub(super) fn discover_immediate_mermaid_files(
    module_dir: &Path,
) -> Result<Vec<PathBuf>, QianjiError> {
    let mut mermaid_files = fs::read_dir(module_dir)
        .map_err(|error| {
            QianjiError::Topology(format!(
                "Failed to read Flowhub module directory `{}`: {error}",
                module_dir.display()
            ))
        })?
        .map(|entry| mermaid_file_path(module_dir, entry))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
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

    WalkDir::new(module_dir)
        .into_iter()
        .try_fold(0_usize, |match_count, entry| {
            let entry = entry.map_err(|error| {
                QianjiError::Topology(format!(
                    "Failed to walk Flowhub module directory `{}`: {error}",
                    module_dir.display()
                ))
            })?;
            if entry.path() == module_dir {
                return Ok(match_count);
            }
            let relative = entry.path().strip_prefix(module_dir).map_err(|error| {
                QianjiError::Topology(format!(
                    "Failed to relativize Flowhub module path `{}` against `{}`: {error}",
                    entry.path().display(),
                    module_dir.display()
                ))
            })?;
            let normalized = relative.to_string_lossy().replace('\\', "/");
            Ok(match_count + usize::from(matcher.is_match(normalized.as_str())))
        })
}

pub(super) fn count_root_glob_matches(root: &Path, pattern: &str) -> Result<usize, QianjiError> {
    let matcher = Glob::new(pattern)
        .map_err(|error| {
            QianjiError::Topology(format!(
                "invalid Flowhub contract glob pattern `{pattern}`: {error}"
            ))
        })?
        .compile_matcher();

    WalkDir::new(root)
        .into_iter()
        .try_fold(0_usize, |match_count, entry| {
            let entry = entry.map_err(|error| {
                QianjiError::Topology(format!(
                    "Failed to walk Flowhub root `{}`: {error}",
                    root.display()
                ))
            })?;
            if entry.path() == root {
                return Ok(match_count);
            }
            let relative = entry.path().strip_prefix(root).map_err(|error| {
                QianjiError::Topology(format!(
                    "Failed to relativize Flowhub root path `{}` against `{}`: {error}",
                    entry.path().display(),
                    root.display()
                ))
            })?;
            let normalized = relative.to_string_lossy().replace('\\', "/");
            Ok(match_count + usize::from(matcher.is_match(normalized.as_str())))
        })
}

fn child_directory_name(
    module_dir: &Path,
    entry: Result<DirEntry, std::io::Error>,
) -> Result<Option<String>, QianjiError> {
    let entry = flowhub_module_entry(module_dir, entry)?;
    let path = entry.path();
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return Ok(None);
    };
    Ok((path.is_dir() && !name.starts_with('.')).then(|| name.to_string()))
}

fn mermaid_file_path(
    module_dir: &Path,
    entry: Result<DirEntry, std::io::Error>,
) -> Result<Option<PathBuf>, QianjiError> {
    let entry = flowhub_module_entry(module_dir, entry)?;
    let path = entry.path();
    Ok(
        (path.is_file()
            && path.extension().and_then(|extension| extension.to_str()) == Some("mmd"))
        .then_some(path),
    )
}

fn flowhub_module_entry(
    module_dir: &Path,
    entry: Result<DirEntry, std::io::Error>,
) -> Result<DirEntry, QianjiError> {
    entry.map_err(|error| {
        QianjiError::Topology(format!(
            "Failed to read Flowhub module entry under `{}`: {error}",
            module_dir.display()
        ))
    })
}

pub(super) fn last_module_segment(module_ref: &str) -> &str {
    module_ref.rsplit('/').next().unwrap_or(module_ref)
}

pub(super) fn is_glob_pattern(value: &str) -> bool {
    value
        .chars()
        .any(|character| matches!(character, '*' | '?' | '[' | ']'))
}
