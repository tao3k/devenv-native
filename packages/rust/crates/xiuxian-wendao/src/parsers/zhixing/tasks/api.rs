//! Parser entrypoint for `Zhixing` task line projections.

use std::collections::HashMap;

use crate::parsers::zhixing::tasks::metadata::split_title_and_metadata_fields;
use crate::parsers::zhixing::tasks::types::TaskLineProjection;
use crate::skill_runtime::zhixing::{
    ATTR_JOURNAL_CARRYOVER, ATTR_TIMER_REMINDED, ATTR_TIMER_SCHEDULED,
};

struct ParsedTaskLine {
    title: String,
    is_completed: bool,
    metadata_fields: HashMap<String, String>,
}

struct ParsedTaskMetadata {
    task_id: Option<String>,
    priority: Option<String>,
    carryover: u32,
    scheduled_at: Option<String>,
    reminded: Option<bool>,
}

/// Parse one zhixing agenda task line into structured task metadata.
#[must_use]
pub fn parse_task_projection(line: &str, line_no: usize) -> Option<TaskLineProjection> {
    let parsed = parse_task_line(line)?;
    let metadata = parse_task_metadata(&parsed.metadata_fields, line);

    Some(TaskLineProjection {
        title: parsed.title,
        line_no,
        is_completed: parsed.is_completed,
        task_id: metadata.task_id,
        priority: metadata.priority,
        carryover: metadata.carryover,
        scheduled_at: metadata.scheduled_at,
        reminded: metadata.reminded,
    })
}

fn parse_task_line(line: &str) -> Option<ParsedTaskLine> {
    let trimmed = line.trim_start();
    let remainder = trimmed.strip_prefix("- [")?;
    let marker_end = remainder.find(']')?;
    let marker = remainder.get(..marker_end)?.trim();
    let is_completed = marker.eq_ignore_ascii_case("x");
    let title_part = remainder.get(marker_end + 1..)?.trim();
    let (title, metadata_fields) = split_title_and_metadata_fields(title_part);
    let title = title.trim().to_string();
    if title.is_empty() {
        return None;
    }

    Some(ParsedTaskLine {
        title,
        is_completed,
        metadata_fields,
    })
}

fn parse_task_metadata(
    metadata_fields: &HashMap<String, String>,
    line: &str,
) -> ParsedTaskMetadata {
    let carryover = metadata_fields
        .get(ATTR_JOURNAL_CARRYOVER)
        .and_then(|value| value.parse::<u32>().ok())
        .or_else(|| parse_carryover(line))
        .unwrap_or(0);
    let task_id = metadata_fields.get("id").cloned();
    let priority = metadata_fields.get("priority").cloned();
    let scheduled_at = metadata_fields
        .get(ATTR_TIMER_SCHEDULED)
        .cloned()
        .or_else(|| metadata_fields.get("scheduled_at").cloned());
    let reminded = metadata_fields
        .get(ATTR_TIMER_REMINDED)
        .and_then(|value| parse_bool_token(value));

    ParsedTaskMetadata {
        task_id,
        priority,
        carryover,
        scheduled_at,
        reminded,
    }
}

fn parse_carryover(line: &str) -> Option<u32> {
    let marker = "journal:carryover:";
    let (_, tail) = line.split_once(marker)?;
    let digits: String = tail
        .trim_start()
        .chars()
        .take_while(char::is_ascii_digit)
        .collect();
    if digits.is_empty() {
        None
    } else {
        digits.parse::<u32>().ok()
    }
}

fn parse_bool_token(raw: &str) -> Option<bool> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "true" | "1" | "yes" | "on" => Some(true),
        "false" | "0" | "no" | "off" => Some(false),
        _ => None,
    }
}
