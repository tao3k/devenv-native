//! Org reflection-question adapter for memory object inference.

use std::path::Path;

use crate::orgize::read_model::model::AgentOrgTaskListRow;
use xiuxian_memory_engine::{InferredMemoryObject, infer_memory_object_from_reflection};

pub(in crate::orgize::read_model) fn reflection_memory_objects_for_row(
    row: &AgentOrgTaskListRow,
) -> Vec<InferredMemoryObject> {
    let Ok(source) = std::fs::read_to_string(Path::new(row.source_path.as_str())) else {
        return Vec::new();
    };
    let Ok(start) = usize::try_from(row.source_range_start) else {
        return Vec::new();
    };
    let Ok(end) = usize::try_from(row.source_range_end) else {
        return Vec::new();
    };
    if start > end || end > source.len() {
        return Vec::new();
    }
    reflection_memory_objects_from_section(&source[start..end])
}

fn reflection_memory_objects_from_section(section: &str) -> Vec<InferredMemoryObject> {
    let Some(root_level) = section_heading_level(section) else {
        return Vec::new();
    };
    let target_level = root_level + 1;
    let mut in_reflection = false;
    let mut objects = Vec::new();

    for line in section.lines().skip(1) {
        let trimmed = line.trim_start();
        if let Some(level) = heading_level(trimmed) {
            if in_reflection && level <= target_level {
                break;
            }
            in_reflection = level == target_level
                && heading_title(trimmed).is_some_and(reflection_heading_title);
            continue;
        }
        if !in_reflection {
            continue;
        }
        let Some((question, value)) = reflection_question_table_row(trimmed) else {
            continue;
        };
        if let Some(object) = infer_memory_object_from_reflection(question, value) {
            objects.push(object);
        }
    }

    objects
}

fn section_heading_level(section: &str) -> Option<usize> {
    section
        .lines()
        .next()
        .and_then(|line| heading_level(line.trim_start()))
}

fn heading_level(line: &str) -> Option<usize> {
    let level = line
        .chars()
        .take_while(|character| *character == '*')
        .count();
    (level > 0 && line.as_bytes().get(level) == Some(&b' ')).then_some(level)
}

fn heading_title(line: &str) -> Option<&str> {
    let level = heading_level(line)?;
    let mut title = line[level..].trim();
    if let Some(rest) = title
        .split_once(char::is_whitespace)
        .and_then(|(head, rest)| agent_task_todo_keyword(head).then_some(rest.trim()))
    {
        title = rest;
    }
    if title.starts_with("[#") && title.get(3..4) == Some("]") {
        title = title[4..].trim_start();
    }
    Some(strip_org_heading_tags(title).trim())
}

fn agent_task_todo_keyword(value: &str) -> bool {
    matches!(
        value,
        "TODO" | "DOING" | "NEXT" | "WAITING" | "DONE" | "CANCELLED"
    )
}

fn strip_org_heading_tags(title: &str) -> &str {
    let Some((before, after)) = title.rsplit_once(' ') else {
        return title;
    };
    if after.starts_with(':')
        && after.ends_with(':')
        && after.trim_matches(':').split(':').all(|tag| {
            !tag.is_empty()
                && tag
                    .chars()
                    .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '@' || ch == '#')
        })
    {
        before
    } else {
        title
    }
}

fn reflection_heading_title(title: &str) -> bool {
    title.eq_ignore_ascii_case("reflection questions")
        || title.eq_ignore_ascii_case("closure questions")
}

fn reflection_question_table_row(line: &str) -> Option<(&str, &str)> {
    let trimmed = line.trim();
    if !trimmed.starts_with('|') {
        return None;
    }
    let cells = trimmed
        .trim_matches('|')
        .split('|')
        .map(str::trim)
        .collect::<Vec<_>>();
    if cells.len() < 2 {
        return None;
    }
    let question = cells[0];
    let value = cells[1];
    if question.is_empty()
        || value.is_empty()
        || question.eq_ignore_ascii_case("question")
        || org_table_separator(question)
        || org_table_separator(value)
    {
        return None;
    }
    Some((question, value))
}

fn org_table_separator(value: &str) -> bool {
    value
        .chars()
        .all(|character| matches!(character, '-' | '+' | '=' | ' '))
}
