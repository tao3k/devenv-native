use super::filters::should_skip_entry;
use crate::parsers::markdown::is_supported_note;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use walkdir::WalkDir;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub(super) struct LinkGraphFingerprint {
    pub note_count: usize,
    pub latest_modified_ts: Option<i64>,
    pub total_size_bytes: u64,
}

fn system_time_to_unix(ts: SystemTime) -> Option<i64> {
    let seconds = ts.duration_since(UNIX_EPOCH).ok()?.as_secs();
    i64::try_from(seconds).ok()
}

fn update_fingerprint(path: &Path, fingerprint: &mut LinkGraphFingerprint) {
    let Ok(meta) = std::fs::metadata(path) else {
        return;
    };
    fingerprint.note_count = fingerprint.note_count.saturating_add(1);
    fingerprint.total_size_bytes = fingerprint.total_size_bytes.saturating_add(meta.len());
    let modified = meta.modified().ok().and_then(system_time_to_unix);
    if let Some(ts) = modified {
        fingerprint.latest_modified_ts =
            Some(fingerprint.latest_modified_ts.map_or(ts, |v| v.max(ts)));
    }
}

pub(super) fn scan_note_fingerprint(
    root: &Path,
    include_dirs: &HashSet<String>,
    excluded_dirs: &HashSet<String>,
) -> LinkGraphFingerprint {
    let mut fingerprint = LinkGraphFingerprint::default();
    for scan_root in fingerprint_scan_roots(root, include_dirs) {
        scan_note_fingerprint_root(
            root,
            &scan_root,
            include_dirs,
            excluded_dirs,
            &mut fingerprint,
        );
    }
    fingerprint
}

fn scan_note_fingerprint_root(
    root: &Path,
    scan_root: &Path,
    include_dirs: &HashSet<String>,
    excluded_dirs: &HashSet<String>,
    fingerprint: &mut LinkGraphFingerprint,
) {
    for entry in WalkDir::new(scan_root)
        .follow_links(false)
        .into_iter()
        .filter_entry(|entry| {
            !should_skip_entry(
                entry.path(),
                entry.file_type().is_dir(),
                root,
                include_dirs,
                excluded_dirs,
            )
        })
        .filter_map(Result::ok)
    {
        let path = entry.path();
        if !entry.file_type().is_file() || !is_supported_note(path) {
            continue;
        }
        update_fingerprint(path, fingerprint);
    }
}

fn fingerprint_scan_roots(root: &Path, include_dirs: &HashSet<String>) -> Vec<PathBuf> {
    if include_dirs.is_empty() {
        return vec![root.to_path_buf()];
    }

    let mut include_roots = include_dirs
        .iter()
        .filter_map(|include_dir| {
            let path = root.join(include_dir);
            path.is_dir().then_some(path)
        })
        .collect::<Vec<_>>();
    include_roots.sort();
    include_roots.dedup();

    let mut scan_roots = Vec::with_capacity(include_roots.len());
    'candidate: for candidate in include_roots {
        for accepted in &scan_roots {
            if candidate.starts_with(accepted) {
                continue 'candidate;
            }
        }
        scan_roots.push(candidate);
    }

    scan_roots
}

#[cfg(test)]
#[path = "../../../../tests/unit/link_graph/index/build/fingerprint.rs"]
mod tests;
