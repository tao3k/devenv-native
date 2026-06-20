use std::path::Path;

use super::ProjectScannedFile;

#[derive(Debug, Clone)]
pub(crate) struct SourceSnapshotEntry {
    pub(crate) content: String,
}

#[must_use]
pub(crate) fn source_snapshot_entry_cache_key(
    project_root: &Path,
    file: &ProjectScannedFile,
) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(project_root.to_string_lossy().as_bytes());
    hasher.update(file.scan_root.to_string_lossy().as_bytes());
    hasher.update(file.partition_id.as_bytes());
    hasher.update(file.absolute_path.to_string_lossy().as_bytes());
    hasher.update(file.normalized_path.as_bytes());
    hasher.update(file.project_name.as_deref().unwrap_or_default().as_bytes());
    hasher.update(file.root_label.as_deref().unwrap_or_default().as_bytes());
    hasher.update(&file.size_bytes.to_le_bytes());
    hasher.update(&file.modified_secs.to_le_bytes());
    hasher.update(&u64::from(file.modified_nanos).to_le_bytes());
    hasher.finalize().to_hex().to_string()
}

#[must_use]
pub(crate) fn build_source_snapshot_entry(file: &ProjectScannedFile) -> SourceSnapshotEntry {
    let Ok(content) = std::fs::read_to_string(file.absolute_path.as_path()) else {
        return SourceSnapshotEntry {
            content: String::new(),
        };
    };

    SourceSnapshotEntry { content }
}
