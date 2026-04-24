use crate::entity::{Entity, EntityType};
use crate::skill_runtime::zhixing::{Error, Result};
use serde_json::json;
use std::fs;
use std::path::Path;

use super::file_discovery::collect_markdown_files;
use super::stats::{count_agenda_statuses, count_reflection_sections};
use super::{ZhixingIndexSummary, ZhixingWendaoIndexer};

impl ZhixingWendaoIndexer {
    pub(super) fn index_document_dir(
        &self,
        segment: &str,
        kind: &str,
        summary: &mut ZhixingIndexSummary,
    ) -> Result<usize> {
        let dir = self.notebook_root.join(segment);
        let files = collect_markdown_files(&dir);
        for file in &files {
            let date = file
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown");
            self.sync_document_path(segment, kind, file, date, summary)?;
        }
        Ok(files.len())
    }

    pub(in crate::skill_runtime::zhixing::indexer) fn sync_document_path(
        &self,
        segment: &str,
        kind: &str,
        file: &Path,
        date: &str,
        summary: &mut ZhixingIndexSummary,
    ) -> Result<bool> {
        let entity_id = document_entity_id(segment, date);
        if !file.exists() {
            if self.graph.get_entity(&entity_id).is_none() {
                return Ok(false);
            }
            self.graph
                .remove_entity(&entity_id)
                .map_err(|error| Error::Internal(format!("Graph operation failed: {error}")))?;
            return Ok(true);
        }

        let content = fs::read_to_string(file).map_err(|error| {
            Error::Internal(format!("Failed reading {}: {error}", file.display()))
        })?;
        let mut entity = Entity::new(
            entity_id,
            format!("{kind} {date}"),
            EntityType::Document,
            content.clone(),
        );
        entity.source = Some(file.display().to_string());
        entity
            .metadata
            .insert("zhixing_domain".to_string(), json!(segment));
        entity.metadata.insert("date".to_string(), json!(date));
        if segment == "journal" {
            entity.metadata.insert(
                "reflection_sections".to_string(),
                json!(count_reflection_sections(&content)),
            );
        }
        if segment == "agenda" {
            let (open_tasks, done_tasks) = count_agenda_statuses(&content);
            entity
                .metadata
                .insert("open_task_count".to_string(), json!(open_tasks));
            entity
                .metadata
                .insert("done_task_count".to_string(), json!(done_tasks));
        }
        if self
            .graph
            .add_entity(entity)
            .map_err(|error| Error::Internal(format!("Graph operation failed: {error}")))?
        {
            summary.entities_added = summary.entities_added.saturating_add(1);
        }
        Ok(true)
    }
}

pub(in crate::skill_runtime::zhixing::indexer) fn document_entity_id(
    segment: &str,
    date: &str,
) -> String {
    format!("zhixing:{segment}:{date}")
}
