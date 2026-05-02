use crate::link_graph::index::build::filters::{
    is_supported_note_candidate, normalized_relative_note_alias, should_skip_entry,
};
use crate::link_graph::index::build::graphmem::sync_graphmem_state_best_effort;
use crate::link_graph::index::{
    INCREMENTAL_REBUILD_THRESHOLD, LinkGraphIndex, LinkGraphRefreshMode,
};
use crate::parsers::markdown::{ParsedNote, is_supported_note, normalize_alias, parse_note};
use std::collections::HashSet;
use std::path::PathBuf;

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
            *self = self.rebuild_from_current_filters()?;
            sync_graphmem_state_best_effort(self);
            return Ok(LinkGraphRefreshMode::Full);
        }

        let included: HashSet<String> = self.include_dirs.iter().cloned().collect();
        let excluded: HashSet<String> = self.excluded_dirs.iter().cloned().collect();
        let mut parsed_updates: Vec<ParsedNote> = Vec::new();
        for changed in changed_paths {
            let raw_candidate = if changed.is_absolute() {
                changed.clone()
            } else {
                self.root.join(changed)
            };
            let candidate = if raw_candidate.exists() {
                raw_candidate
                    .canonicalize()
                    .unwrap_or_else(|_| raw_candidate.clone())
            } else {
                raw_candidate
            };
            if should_skip_entry(&candidate, false, &self.root, &included, &excluded) {
                continue;
            }
            if !is_supported_note_candidate(&candidate) {
                continue;
            }

            if let Some(alias) = normalized_relative_note_alias(&candidate, &self.root)
                && let Some(existing_id) = self
                    .resolve_doc_id(&alias)
                    .map(std::string::ToString::to_string)
            {
                self.remove_doc_by_id(&existing_id);
            } else if let Some(stem) = candidate.file_stem().and_then(|v| v.to_str()) {
                let stem_alias = normalize_alias(stem);
                if let Some(existing_id) = self
                    .resolve_doc_id(&stem_alias)
                    .map(std::string::ToString::to_string)
                {
                    self.remove_doc_by_id(&existing_id);
                }
            }

            if !candidate.exists() || !candidate.is_file() {
                continue;
            }
            if !is_supported_note(&candidate) {
                continue;
            }
            let content = std::fs::read_to_string(&candidate).map_err(|e| {
                format!("failed to read changed note '{}': {e}", candidate.display())
            })?;
            if let Some(parsed) = parse_note(&candidate, &self.root, &content) {
                parsed_updates.push(parsed);
            }
        }

        for parsed in &parsed_updates {
            self.insert_doc_no_edges(parsed);
        }
        for parsed in &parsed_updates {
            self.add_outgoing_links_for_doc(parsed);
        }
        self.prune_empty_edge_sets();
        self.recompute_edge_count();
        self.recompute_rank_by_id();
        sync_graphmem_state_best_effort(self);
        Ok(LinkGraphRefreshMode::Delta)
    }
}
