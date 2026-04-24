use crate::entity::{Entity, EntityType};
use crate::parsers::zhixing::tasks::{
    TaskLineProjection, normalize_identity_token, parse_task_projection,
};
use crate::skill_runtime::zhixing::{
    ATTR_JOURNAL_CARRYOVER, ATTR_TIMER_REMINDED, ATTR_TIMER_SCHEDULED, Error, Result,
};
use serde_json::json;
use std::fs;
use std::path::Path;

use super::super::file_discovery::collect_markdown_files;
use super::super::{ZhixingIndexSummary, ZhixingWendaoIndexer};

impl ZhixingWendaoIndexer {
    pub(in crate::skill_runtime::zhixing) fn index_agenda_tasks(
        &self,
        summary: &mut ZhixingIndexSummary,
    ) -> Result<usize> {
        let agenda_dir = self.notebook_root.join("agenda");
        let files = collect_markdown_files(&agenda_dir);
        let mut indexed = 0usize;

        for file in files {
            let content = fs::read_to_string(&file).map_err(|error| {
                Error::Internal(format!("Failed reading {}: {error}", file.display()))
            })?;
            let date = file
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown");
            let agenda_entity_name = format!("Agenda {date}");
            for (line_no, line) in content.lines().enumerate() {
                let Some(task) = parse_task_projection(line, line_no + 1) else {
                    continue;
                };
                let (entity, task_entity_name) = build_task_entity(&file, date, &task);
                if self
                    .graph
                    .add_entity(entity)
                    .map_err(|error| Error::Internal(format!("Graph operation failed: {error}")))?
                {
                    summary.entities_added = summary.entities_added.saturating_add(1);
                }
                self.link_task_to_agenda_document(
                    &agenda_entity_name,
                    &task_entity_name,
                    &file,
                    task.line_no,
                )?;
                summary.relations_linked = summary.relations_linked.saturating_add(1);
                indexed = indexed.saturating_add(1);
            }
        }
        Ok(indexed)
    }

    pub(in crate::skill_runtime::zhixing) fn reindex_agenda_tasks_for_path(
        &self,
        file: &Path,
        date: &str,
        summary: &mut ZhixingIndexSummary,
    ) -> Result<usize> {
        let agenda_entity_name = format!("Agenda {date}");
        let source_path = file.display().to_string();

        let task_ids = self
            .graph
            .get_entities_by_type("TASK")
            .into_iter()
            .filter(|entity| {
                entity.metadata.get("agenda_date") == Some(&json!(date))
                    && entity.source.as_deref() == Some(source_path.as_str())
            })
            .map(|entity| entity.id)
            .collect::<Vec<_>>();

        for task_id in task_ids {
            self.graph
                .remove_entity(&task_id)
                .map_err(|error| Error::Internal(format!("Graph operation failed: {error}")))?;
        }

        // 1. Remove existing task links for this agenda document
        self.graph.remove_relations_for_source(&agenda_entity_name);

        if !file.exists() {
            return Ok(0);
        }

        let content = fs::read_to_string(file).map_err(|error| {
            Error::Internal(format!("Failed reading {}: {error}", file.display()))
        })?;

        let mut indexed = 0usize;
        for (line_no, line) in content.lines().enumerate() {
            let Some(task) = parse_task_projection(line, line_no + 1) else {
                continue;
            };
            let (entity, task_entity_name) = build_task_entity(file, date, &task);
            if self
                .graph
                .add_entity(entity)
                .map_err(|error| Error::Internal(format!("Graph operation failed: {error}")))?
            {
                summary.entities_added = summary.entities_added.saturating_add(1);
            }
            self.link_task_to_agenda_document(
                &agenda_entity_name,
                &task_entity_name,
                file,
                task.line_no,
            )?;
            summary.relations_linked = summary.relations_linked.saturating_add(1);
            indexed = indexed.saturating_add(1);
        }
        Ok(indexed)
    }
}

fn build_task_entity(
    source_file: &Path,
    date: &str,
    task: &TaskLineProjection,
) -> (Entity, String) {
    let display_key = task
        .task_id
        .clone()
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| format!("line-{}", task.line_no));
    let stable_token = normalize_identity_token(&display_key);
    let task_entity_name = format!("{} [{date}:{display_key}]", task.title);

    let mut entity = Entity::new(
        format!("zhixing:task:{date}:{stable_token}"),
        task_entity_name.clone(),
        EntityType::Other("Task".to_string()),
        task.title.clone(),
    );
    entity.source = Some(source_file.display().to_string());
    entity.metadata.insert(
        ATTR_JOURNAL_CARRYOVER.to_string(),
        json!(u64::from(task.carryover)),
    );
    entity
        .metadata
        .insert("agenda_date".to_string(), json!(date));
    entity
        .metadata
        .insert("source_line".to_string(), json!(task.line_no));
    entity
        .metadata
        .insert("zhixing_domain".to_string(), json!("task"));
    entity.metadata.insert(
        "task_status".to_string(),
        json!(if task.is_completed { "done" } else { "todo" }),
    );
    entity
        .metadata
        .insert("is_completed".to_string(), json!(task.is_completed));

    if let Some(task_id) = task.task_id.clone() {
        entity
            .metadata
            .insert("task_id".to_string(), json!(task_id));
    }
    if let Some(priority) = task.priority.clone() {
        entity
            .metadata
            .insert("task_priority".to_string(), json!(priority));
    }
    if let Some(scheduled_at) = task.scheduled_at.clone() {
        entity
            .metadata
            .insert(ATTR_TIMER_SCHEDULED.to_string(), json!(scheduled_at));
    }
    if let Some(reminded) = task.reminded {
        entity
            .metadata
            .insert(ATTR_TIMER_REMINDED.to_string(), json!(reminded));
    }

    (entity, task_entity_name)
}
