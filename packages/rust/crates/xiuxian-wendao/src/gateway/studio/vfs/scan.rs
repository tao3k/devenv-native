use std::fs;
use std::path::Path;
use std::time::Instant;

use walkdir::{DirEntry, WalkDir};

use crate::gateway::studio::router::StudioState;
use crate::gateway::studio::types::{VfsScanEntry, VfsScanResult};

use super::categories::guess_category;
use super::filters::ProjectFileFilter;
use super::roots::resolve_all_vfs_roots;

struct VfsCounters {
    files: usize,
    dirs: usize,
}

pub(crate) fn scan_all_roots(state: &StudioState) -> VfsScanResult {
    let start = Instant::now();
    let mut entries = Vec::new();
    let mut counters = VfsCounters { files: 0, dirs: 0 };

    let roots = resolve_all_vfs_roots(state);

    for root in roots {
        scan_directory(
            &root.full_path,
            root.project_name.as_deref(),
            root.root_label.as_deref(),
            root.request_root.as_str(),
            root.filter_prefix.as_str(),
            &mut counters,
            &root.file_filters,
            &mut entries,
        );
    }

    VfsScanResult {
        entries,
        file_count: counters.files,
        dir_count: counters.dirs,
        scan_duration_ms: elapsed_millis_u64(start),
    }
}

pub(crate) fn scan_roots(state: &StudioState) -> VfsScanResult {
    if let Some(existing) = state
        .vfs_scan
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .as_ref()
    {
        return existing.clone();
    }

    let result = scan_all_roots(state);
    let mut guard = state
        .vfs_scan
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if let Some(existing) = guard.as_ref() {
        return existing.clone();
    }
    *guard = Some(result.clone());
    result
}

fn elapsed_millis_u64(start: Instant) -> u64 {
    u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX)
}

#[allow(clippy::too_many_arguments)]
fn scan_directory(
    dir_path: &Path,
    project_name: Option<&str>,
    root_label: Option<&str>,
    request_root: &str,
    filter_prefix: &str,
    counters: &mut VfsCounters,
    filters: &[ProjectFileFilter],
    entries: &mut Vec<VfsScanEntry>,
) {
    let walk = WalkDir::new(dir_path)
        .max_depth(10)
        .into_iter()
        .filter_entry(|entry| !should_skip_entry(entry));

    for entry in walk.flatten() {
        let path = entry.path();
        if !filters.iter().any(|filter| filter.matches(path)) {
            continue;
        }

        let metadata = entry.metadata().ok();
        let size = metadata.as_ref().map_or(0, fs::Metadata::len);
        let modified = metadata
            .as_ref()
            .and_then(|value| value.modified().ok())
            .and_then(|value| value.duration_since(std::time::UNIX_EPOCH).ok())
            .map_or(0, |duration| duration.as_secs());

        let is_dir = entry.file_type().is_dir();
        if is_dir {
            counters.dirs += 1;
        } else {
            counters.files += 1;
        }

        let rel_path = path.strip_prefix(dir_path).unwrap_or(path);
        let display_path = if filter_prefix.is_empty() {
            format!("{}/{}", request_root, rel_path.display())
        } else {
            format!("{}/{}/{}", request_root, filter_prefix, rel_path.display())
        };

        entries.push(VfsScanEntry {
            path: display_path.replace('\\', "/"),
            name: entry.file_name().to_string_lossy().to_string(),
            is_dir,
            category: guess_category(entry.path()),
            size,
            modified,
            content_type: None,
            has_frontmatter: false,
            wendao_id: None,
            project_name: project_name.map(String::from),
            root_label: root_label.map(String::from),
            project_root: None,
            project_dirs: None,
        });
    }
}

fn should_skip_entry(entry: &DirEntry) -> bool {
    let name = entry.file_name().to_string_lossy();
    name.starts_with('.') || name == "target" || name == "node_modules"
}
