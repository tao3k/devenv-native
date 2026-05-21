//! Incremental refresh application for link-graph indexes.

use crate::link_graph::index::build::filters::{
    is_supported_note_candidate, normalized_relative_note_alias, should_skip_entry,
};
use crate::link_graph::index::build::graphmem::sync_graphmem_state_best_effort;
use crate::link_graph::index::{
    INCREMENTAL_REBUILD_THRESHOLD, LinkGraphIndex, LinkGraphRefreshMode,
};
use crate::parsers::markdown::{ParsedNote, is_supported_note, normalize_alias, parse_note};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

struct IncrementalRefreshScope {
    included: HashSet<String>,
    excluded: HashSet<String>,
}

impl IncrementalRefreshScope {
    fn from_index(index: &LinkGraphIndex) -> Self {
        Self {
            included: index.include_dirs.iter().cloned().collect(),
            excluded: index.excluded_dirs.iter().cloned().collect(),
        }
    }
}

impl LinkGraphIndex {
    /// Apply incremental updates for changed note files.
    ///
    /// Falls back to full rebuild when change-set is large.
    ///
    /// # Errors
    ///
    /// Returns an error when incremental or fallback rebuild operations fail.
    pub fn refresh_incremental(&mut self, changed_paths: &[PathBuf]) -> Result<(), String> {
        let _ =
            self.refresh_incremental_with_threshold(changed_paths, INCREMENTAL_REBUILD_THRESHOLD)?;
        Ok(())
    }

    /// Apply incremental updates for changed note files with explicit threshold.
    ///
    /// # Errors
    ///
    /// Returns an error when full rebuild or changed-file read operations fail.
    pub fn refresh_incremental_with_threshold(
        &mut self,
        changed_paths: &[PathBuf],
        full_rebuild_threshold: usize,
    ) -> Result<LinkGraphRefreshMode, String> {
        if changed_paths.is_empty() {
            return Ok(LinkGraphRefreshMode::Noop);
        }
        let threshold = full_rebuild_threshold.max(1);
        if changed_paths.len() >= threshold {
            return self.apply_full_refresh();
        }

        self.apply_delta_refresh(changed_paths)
    }

    fn apply_full_refresh(&mut self) -> Result<LinkGraphRefreshMode, String> {
        *self = self.rebuild_from_current_filters()?;
        sync_graphmem_state_best_effort(self);
        Ok(LinkGraphRefreshMode::Full)
    }

    fn apply_delta_refresh(
        &mut self,
        changed_paths: &[PathBuf],
    ) -> Result<LinkGraphRefreshMode, String> {
        let scope = IncrementalRefreshScope::from_index(self);
        let parsed_updates = self.parse_changed_notes(changed_paths, &scope)?;
        self.apply_parsed_updates(&parsed_updates);
        sync_graphmem_state_best_effort(self);
        Ok(LinkGraphRefreshMode::Delta)
    }

    fn parse_changed_notes(
        &mut self,
        changed_paths: &[PathBuf],
        scope: &IncrementalRefreshScope,
    ) -> Result<Vec<ParsedNote>, String> {
        changed_paths
            .iter()
            .try_fold(Vec::new(), |mut updates, changed| {
                if let Some(parsed) = self.parse_changed_path(changed, scope)? {
                    updates.push(parsed);
                }
                Ok(updates)
            })
    }

    fn parse_changed_path(
        &mut self,
        changed: &Path,
        scope: &IncrementalRefreshScope,
    ) -> Result<Option<ParsedNote>, String> {
        let candidate = self.changed_candidate(changed);
        if self.should_skip_changed_candidate(&candidate, scope) {
            return Ok(None);
        }
        self.remove_existing_doc_for_candidate(&candidate);
        self.parse_changed_note(&candidate)
    }

    fn changed_candidate(&self, changed: &Path) -> PathBuf {
        let raw_candidate = if changed.is_absolute() {
            changed.to_path_buf()
        } else {
            self.root.join(changed)
        };
        if raw_candidate.exists() {
            raw_candidate
                .canonicalize()
                .unwrap_or_else(|_| raw_candidate.clone())
        } else {
            raw_candidate
        }
    }

    fn should_skip_changed_candidate(
        &self,
        candidate: &Path,
        scope: &IncrementalRefreshScope,
    ) -> bool {
        should_skip_entry(
            candidate,
            false,
            &self.root,
            &scope.included,
            &scope.excluded,
        ) || !is_supported_note_candidate(candidate)
    }

    fn remove_existing_doc_for_candidate(&mut self, candidate: &Path) {
        if let Some(existing_id) = self.existing_doc_id_for_candidate(candidate) {
            self.remove_doc_by_id(&existing_id);
        }
    }

    fn existing_doc_id_for_candidate(&self, candidate: &Path) -> Option<String> {
        normalized_relative_note_alias(candidate, &self.root)
            .and_then(|alias| self.resolve_doc_id(&alias))
            .or_else(|| {
                candidate
                    .file_stem()
                    .and_then(|value| value.to_str())
                    .map(normalize_alias)
                    .and_then(|alias| self.resolve_doc_id(&alias))
            })
            .map(std::string::ToString::to_string)
    }

    fn parse_changed_note(&self, candidate: &Path) -> Result<Option<ParsedNote>, String> {
        if !candidate.exists() || !candidate.is_file() || !is_supported_note(candidate) {
            return Ok(None);
        }
        let content = std::fs::read_to_string(candidate).map_err(|error| {
            format!(
                "failed to read changed note '{}': {error}",
                candidate.display()
            )
        })?;
        Ok(parse_note(candidate, &self.root, &content))
    }

    fn apply_parsed_updates(&mut self, parsed_updates: &[ParsedNote]) {
        self.insert_parsed_docs(parsed_updates);
        self.add_parsed_doc_edges(parsed_updates);
        self.finalize_delta_refresh_graph();
    }

    fn insert_parsed_docs(&mut self, parsed_updates: &[ParsedNote]) {
        for parsed in parsed_updates {
            self.insert_doc_no_edges(parsed);
        }
    }

    fn add_parsed_doc_edges(&mut self, parsed_updates: &[ParsedNote]) {
        for parsed in parsed_updates {
            self.add_outgoing_links_for_doc(parsed);
        }
    }

    fn finalize_delta_refresh_graph(&mut self) {
        self.prune_empty_edge_sets();
        self.recompute_edge_count();
        self.recompute_rank_by_id();
    }
}
