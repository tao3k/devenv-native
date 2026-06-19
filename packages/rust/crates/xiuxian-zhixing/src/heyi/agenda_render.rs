use super::ZhixingHeyi;
use super::constants::{ATTR_JOURNAL_CARRYOVER, ATTR_TIMER_SCHEDULED};
use super::metadata::parse_carryover_count;
use super::schedule_time::render_scheduled_time_local;
use crate::Result;
use chrono::Utc;
use serde_json::json;
use std::fs;
use xiuxian_wendao::LinkGraphIndex;
use xiuxian_wendao::link_graph::{LinkGraphHit, LinkGraphSearchOptions};

const JOURNAL_INCLUDE_PATH: &str = "journal/";
const AGENDA_SEARCH_LIMIT: usize = 8;
const TODAY_AGENDA_QUERY_HINT: &str = "today 今日 agenda 日程 议程";

fn strip_html_comment_lines(text: &str) -> String {
    text.lines()
        .filter(|line| {
            let trimmed = line.trim();
            !(trimmed.starts_with("<!--") && trimmed.ends_with("-->"))
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn select_today_journal_hit<'a>(
    hits: &'a [LinkGraphHit],
    local_date: &str,
) -> Option<&'a LinkGraphHit> {
    let target = format!("journal/{local_date}.md");
    hits.iter()
        .find(|hit| hit.path == target || hit.path.ends_with(&target))
        .or_else(|| {
            hits.iter()
                .find(|hit| hit.path.starts_with(JOURNAL_INCLUDE_PATH))
        })
}

impl ZhixingHeyi {
    fn render_agenda_from_wendao_today_note(&self) -> Option<String> {
        let index = match LinkGraphIndex::build(&self.storage.root_dir) {
            Ok(value) => value,
            Err(error) => {
                log::warn!("wendao agenda view build failed: {error}");
                return None;
            }
        };

        let local_date = Utc::now()
            .with_timezone(&self.time_zone)
            .format("%Y-%m-%d")
            .to_string();
        let mut options = LinkGraphSearchOptions::default();
        options.filters.include_paths = vec![JOURNAL_INCLUDE_PATH.to_string()];
        let query = format!("{TODAY_AGENDA_QUERY_HINT} {local_date}");
        let (_parsed, hits) = index.search_planned(&query, AGENDA_SEARCH_LIMIT, options);
        let hit = select_today_journal_hit(&hits, &local_date)?;

        let note_path = self.storage.root_dir.join(&hit.path);
        let note_content = match fs::read_to_string(&note_path) {
            Ok(value) => value,
            Err(error) => {
                log::warn!(
                    "wendao agenda view read failed for path='{}': {error}",
                    note_path.display()
                );
                return None;
            }
        };
        let note_content = note_content.trim();
        if note_content.is_empty() {
            return None;
        }
        let cleaned_note = strip_html_comment_lines(note_content).trim().to_string();
        if cleaned_note.is_empty() {
            return None;
        }
        Some(format!("# Daily Agenda ({local_date})\n\n{cleaned_note}"))
    }

    /// Renders the current cultivation agenda with local time context.
    ///
    /// # Errors
    /// Returns an error when template rendering fails.
    pub fn render_agenda(&self) -> Result<String> {
        self.check_heart_demon_blocker()?;
        if let Some(rendered) = self.render_agenda_from_wendao_today_note() {
            return Ok(rendered);
        }

        let tasks = self.graph.get_entities_by_type("TASK");
        let mut active_tasks = Vec::with_capacity(tasks.len());
        let mut max_carryover = 0;

        for entity in tasks {
            let carryover = entity
                .metadata
                .get(ATTR_JOURNAL_CARRYOVER)
                .and_then(parse_carryover_count)
                .unwrap_or(0);
            if carryover > max_carryover {
                max_carryover = carryover;
            }
            let scheduled_local = entity
                .metadata
                .get(ATTR_TIMER_SCHEDULED)
                .and_then(serde_json::Value::as_str)
                .map_or_else(
                    || "Unscheduled".to_string(),
                    |value| render_scheduled_time_local(value, self.time_zone),
                );

            active_tasks.push(json!({
                "title": entity.name,
                "heat": 0.85,
                "carryover": carryover,
                "scheduled_local": scheduled_local,
            }));
        }

        let template_data = json!({
            "active_tasks": active_tasks,
            "time_zone": self.time_zone.to_string(),
            "requires_review": max_carryover >= 3,
        });

        self.render_with_manifestation_context("daily_agenda.md", template_data, "SUCCESS_STREAK")
    }
}
