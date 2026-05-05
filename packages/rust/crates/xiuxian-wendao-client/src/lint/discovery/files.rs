//! File collection helpers for markdown lint scans.

use std::collections::BTreeSet;
use std::path::Component;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};
use walkdir::{DirEntry, WalkDir};

use super::MarkdownLintArgs;
use super::config;

const TRANSIENT_REPO_DIRS: &[&str] = &[
    ".git",
    ".devenv",
    ".direnv",
    ".cache",
    ".config",
    ".data",
    ".run",
    ".bin",
    "node_modules",
    "target",
];

pub(crate) fn collect_markdown_files(root: &Path, args: &MarkdownLintArgs) -> Result<Vec<PathBuf>> {
    let roots = resolve_scan_roots(root, args)?;
    let mut files = BTreeSet::new();
    for scan_root in roots {
        let metadata = std::fs::metadata(&scan_root).with_context(|| {
            format!(
                "failed to read metadata for markdown lint path `{}`",
                scan_root.display()
            )
        })?;
        if metadata.is_file() {
            if is_markdown_path(&scan_root) {
                files.insert(scan_root);
            }
            continue;
        }
        if !metadata.is_dir() {
            return Err(anyhow!(
                "markdown lint path `{}` is neither a file nor a directory",
                scan_root.display()
            ));
        }
        walk_markdown_dir(&scan_root, args, &mut files)?;
    }
    Ok(files.into_iter().collect())
}

pub(crate) fn display_path(path: &Path, root: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn resolve_scan_roots(root: &Path, args: &MarkdownLintArgs) -> Result<Vec<PathBuf>> {
    if args.paths.is_empty() {
        return config::configured_project_roots(root);
    }

    Ok(args
        .paths
        .iter()
        .map(|path| resolve_path(root, path))
        .collect::<Vec<_>>())
}

fn walk_markdown_dir(
    scan_root: &Path,
    args: &MarkdownLintArgs,
    files: &mut BTreeSet<PathBuf>,
) -> Result<()> {
    let walker = WalkDir::new(scan_root)
        .into_iter()
        .filter_entry(|entry| !should_skip_entry(entry, scan_root, args));
    for entry in walker {
        let entry = entry.with_context(|| {
            format!(
                "failed to walk markdown lint path `{}`",
                scan_root.display()
            )
        })?;
        if entry.file_type().is_file() && is_markdown_path(entry.path()) {
            files.insert(entry.path().to_path_buf());
        }
    }
    Ok(())
}

fn should_skip_entry(entry: &DirEntry, scan_root: &Path, args: &MarkdownLintArgs) -> bool {
    if entry.path() == scan_root {
        return false;
    }
    entry.file_type().is_dir()
        && entry
            .file_name()
            .to_str()
            .is_some_and(|name| should_skip_dir(name, args))
}

fn should_skip_dir(name: &str, args: &MarkdownLintArgs) -> bool {
    transient_repo_dir_name(name).is_some()
        || args
            .skip_dirs
            .iter()
            .any(|skip_dir| skip_dir.as_str() == name)
}

fn transient_repo_dir_name(name: &str) -> Option<&'static str> {
    TRANSIENT_REPO_DIRS
        .iter()
        .copied()
        .find(|candidate| *candidate == name)
}

pub(crate) fn first_transient_repo_dir(path: &Path) -> Option<&'static str> {
    path.components().find_map(|component| match component {
        Component::Normal(name) => name.to_str().and_then(transient_repo_dir_name),
        _ => None,
    })
}

fn resolve_path(root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    }
}

fn is_markdown_path(path: &Path) -> bool {
    matches!(
        path.extension().and_then(std::ffi::OsStr::to_str),
        Some("md" | "markdown")
    )
}
