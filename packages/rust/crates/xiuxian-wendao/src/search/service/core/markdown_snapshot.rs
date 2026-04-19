use std::collections::BTreeMap;
use std::sync::Arc;

use super::types::SearchPlaneService;
use crate::parsers::markdown::is_supported_note;
use crate::search::{
    MarkdownProjectSnapshot, ProjectScannedFile, build_markdown_snapshot_entry,
    markdown_snapshot_entry_cache_key,
};

impl SearchPlaneService {
    #[must_use]
    pub(crate) fn shared_markdown_project_snapshot(
        &self,
        project_root: &std::path::Path,
        files: &[ProjectScannedFile],
    ) -> MarkdownProjectSnapshot {
        let mut entries_by_path = BTreeMap::new();
        for file in files {
            if !is_supported_note(file.absolute_path.as_path()) {
                continue;
            }
            let entry = self.shared_markdown_snapshot_entry(project_root, file);
            entries_by_path.insert(file.normalized_path.clone(), entry);
        }
        MarkdownProjectSnapshot::new(entries_by_path)
    }

    #[must_use]
    pub(crate) fn shared_markdown_snapshot_entry(
        &self,
        project_root: &std::path::Path,
        file: &ProjectScannedFile,
    ) -> Arc<crate::search::MarkdownSnapshotEntry> {
        let cache_key = markdown_snapshot_entry_cache_key(project_root, file);
        let cell = self
            .markdown_snapshot_entries
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(&cache_key)
            .cloned()
            .unwrap_or_else(|| {
                self.markdown_snapshot_entries
                    .write()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .entry(cache_key)
                    .or_insert_with(|| Arc::new(std::sync::OnceLock::new()))
                    .clone()
            });
        if let Some(existing) = cell.get() {
            self.record_repeat_work_file(
                "markdown_snapshot.cache",
                "cache_hit",
                file.normalized_path.as_str(),
            );
            return Arc::clone(existing);
        }

        let mut built_now = false;
        let entry = cell.get_or_init(|| {
            built_now = true;
            Arc::new(build_markdown_snapshot_entry(project_root, file))
        });
        if built_now {
            self.record_repeat_work_file(
                "markdown_snapshot.cache",
                "cache_miss",
                file.normalized_path.as_str(),
            );
            self.record_repeat_work_file(
                "markdown_snapshot.build",
                "read_parse_compile",
                file.normalized_path.as_str(),
            );
        } else {
            self.record_repeat_work_file(
                "markdown_snapshot.cache",
                "cache_hit",
                file.normalized_path.as_str(),
            );
        }
        Arc::clone(entry)
    }

    #[cfg(test)]
    #[must_use]
    pub(crate) fn markdown_snapshot_entry_cache_len(&self) -> usize {
        self.markdown_snapshot_entries
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .len()
    }
}
