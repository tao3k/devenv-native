use std::sync::Arc;

use super::types::SearchPlaneService;
use crate::gateway::studio::search::ast_search_lang;
use crate::parsers::markdown::is_supported_note;
use crate::search::{
    ProjectScannedFile, SourceSnapshotEntry, build_source_snapshot_entry,
    source_snapshot_entry_cache_key,
};

impl SearchPlaneService {
    #[must_use]
    pub(crate) fn shared_source_snapshot_entry(
        &self,
        project_root: &std::path::Path,
        file: &ProjectScannedFile,
    ) -> Arc<SourceSnapshotEntry> {
        if is_supported_note(file.absolute_path.as_path())
            || ast_search_lang(std::path::Path::new(file.normalized_path.as_str())).is_none()
        {
            return Arc::new(SourceSnapshotEntry {
                content: String::new(),
                ast_hits: Vec::new(),
            });
        }

        let cache_key = source_snapshot_entry_cache_key(project_root, file);
        let maybe_cell = {
            self.source_snapshot_entries
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .get(&cache_key)
                .cloned()
        };
        let cell = if let Some(cell) = maybe_cell {
            cell
        } else {
            self.source_snapshot_entries
                .write()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .entry(cache_key)
                .or_insert_with(|| Arc::new(std::sync::OnceLock::new()))
                .clone()
        };
        if let Some(existing) = cell.get() {
            self.record_repeat_work_file(
                "source_snapshot",
                "cache_hit",
                file.normalized_path.as_str(),
            );
            return Arc::clone(existing);
        }

        let mut built_now = false;
        let entry = cell.get_or_init(|| {
            built_now = true;
            Arc::new(build_source_snapshot_entry(file))
        });
        if built_now {
            self.record_repeat_work_file(
                "source_snapshot",
                "cache_miss",
                file.normalized_path.as_str(),
            );
            self.record_repeat_work_file(
                "source_snapshot",
                "read_ast_extract",
                file.normalized_path.as_str(),
            );
        } else {
            self.record_repeat_work_file(
                "source_snapshot",
                "cache_hit",
                file.normalized_path.as_str(),
            );
        }
        Arc::clone(entry)
    }

    #[cfg(test)]
    #[must_use]
    pub(crate) fn source_snapshot_entry_cache_len(&self) -> usize {
        self.source_snapshot_entries
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .len()
    }
}
