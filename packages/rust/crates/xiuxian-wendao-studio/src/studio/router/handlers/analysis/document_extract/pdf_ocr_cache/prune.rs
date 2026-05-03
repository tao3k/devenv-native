use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use super::types::{PdfOcrShardCacheEntry, PdfOcrShardCachePolicy, PdfOcrShardCachePruneReport};

pub(super) fn prune_ocr_shard_cache(
    root: &Path,
    policy: &PdfOcrShardCachePolicy,
) -> Result<PdfOcrShardCachePruneReport, String> {
    let mut entries = collect_ocr_shard_cache_entries(root)?;
    let scanned_entries = entries.len();
    let scanned_bytes = entries.iter().map(|entry| entry.bytes).sum::<u64>();
    let mut report = PdfOcrShardCachePruneReport {
        scanned_entries,
        scanned_bytes,
        ..PdfOcrShardCachePruneReport::default()
    };

    if let Some(max_age) = policy.max_age {
        let now = SystemTime::now();
        entries.retain(|entry| {
            let is_expired = now
                .duration_since(entry.modified)
                .is_ok_and(|age| age > max_age);
            if is_expired && remove_cache_entry(entry, &mut report).is_err() {
                return true;
            }
            !is_expired
        });
    }

    entries.sort_by(|left, right| {
        left.modified
            .cmp(&right.modified)
            .then(left.path.cmp(&right.path))
    });
    enforce_entry_limit(&mut entries, policy.max_entries, &mut report);
    enforce_byte_limit(&mut entries, policy.max_bytes, &mut report);

    report.retained_entries = entries.len();
    report.retained_bytes = entries.iter().map(|entry| entry.bytes).sum();
    Ok(report)
}

fn enforce_entry_limit(
    entries: &mut Vec<PdfOcrShardCacheEntry>,
    max_entries: Option<usize>,
    report: &mut PdfOcrShardCachePruneReport,
) {
    let Some(max_entries) = max_entries else {
        return;
    };
    while entries.len() > max_entries {
        let entry = entries.remove(0);
        if remove_cache_entry(&entry, report).is_err() {
            break;
        }
    }
}

fn enforce_byte_limit(
    entries: &mut Vec<PdfOcrShardCacheEntry>,
    max_bytes: Option<u64>,
    report: &mut PdfOcrShardCachePruneReport,
) {
    let Some(max_bytes) = max_bytes else {
        return;
    };
    let mut retained_bytes = entries.iter().map(|entry| entry.bytes).sum::<u64>();
    while retained_bytes > max_bytes && !entries.is_empty() {
        let entry = entries.remove(0);
        retained_bytes = retained_bytes.saturating_sub(entry.bytes);
        if remove_cache_entry(&entry, report).is_err() {
            break;
        }
    }
}

fn collect_ocr_shard_cache_entries(root: &Path) -> Result<Vec<PdfOcrShardCacheEntry>, String> {
    if !root.exists() {
        return Ok(Vec::new());
    }
    let mut entries = Vec::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        let read_dir = match fs::read_dir(directory.as_path()) {
            Ok(read_dir) => read_dir,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => {
                return Err(format!(
                    "read OCR shard cache directory `{}`: {error}",
                    directory.display()
                ));
            }
        };
        collect_cache_entries_from_directory(
            read_dir,
            directory.as_path(),
            &mut pending,
            &mut entries,
        )?;
    }
    Ok(entries)
}

fn collect_cache_entries_from_directory(
    read_dir: fs::ReadDir,
    directory: &Path,
    pending: &mut Vec<std::path::PathBuf>,
    entries: &mut Vec<PdfOcrShardCacheEntry>,
) -> Result<(), String> {
    for entry in read_dir {
        let entry = entry.map_err(|error| {
            format!(
                "read OCR shard cache entry `{}`: {error}",
                directory.display()
            )
        })?;
        let path = entry.path();
        let metadata = match entry.metadata() {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => {
                return Err(format!(
                    "read OCR shard cache metadata `{}`: {error}",
                    path.display()
                ));
            }
        };
        if metadata.is_dir() {
            pending.push(path);
            continue;
        }
        if path.extension().and_then(|extension| extension.to_str()) != Some("arrow") {
            continue;
        }
        entries.push(PdfOcrShardCacheEntry {
            path,
            bytes: metadata.len(),
            modified: metadata.modified().unwrap_or(UNIX_EPOCH),
        });
    }
    Ok(())
}

fn remove_cache_entry(
    entry: &PdfOcrShardCacheEntry,
    report: &mut PdfOcrShardCachePruneReport,
) -> Result<(), String> {
    match fs::remove_file(entry.path.as_path()) {
        Ok(()) => {
            report.removed_entries += 1;
            report.removed_bytes += entry.bytes;
            Ok(())
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!(
            "remove OCR shard cache entry `{}`: {error}",
            entry.path.display()
        )),
    }
}
