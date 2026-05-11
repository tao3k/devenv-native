//! Markdown section extraction from parsed structure.

use std::collections::HashMap;

use crate::markdown_structure::{MarkdownStructure, parse_markdown_structure};

use super::logbook::extract_logbook_entries;
use super::properties::extract_property_drawers;
use super::types::{MarkdownSection, SectionCore, SectionMetadata, SectionScope};

#[derive(Clone, Copy)]
struct SectionCursor<'a> {
    heading_title: &'a str,
    heading_path: &'a str,
    heading_level: usize,
    line_range: (usize, usize),
    byte_range: (usize, usize),
}

fn section_from_cursor(cursor: SectionCursor<'_>, lines: &[String]) -> Option<MarkdownSection> {
    let section_text = lines.join("\n").trim().to_string();
    if section_text.is_empty() && cursor.heading_path.trim().is_empty() {
        return None;
    }

    let attributes = if cursor.heading_level > 0 {
        extract_property_drawers(lines)
    } else {
        HashMap::new()
    };

    let logbook = if cursor.heading_level > 0 {
        extract_logbook_entries(lines, cursor.line_range.0)
    } else {
        Vec::new()
    };

    let line_start = cursor.line_range.0.max(1);
    let line_end = cursor.line_range.1.max(line_start);

    Some(SectionCore {
        scope: SectionScope {
            heading_title: cursor.heading_title.to_string(),
            heading_path: cursor.heading_path.to_string(),
            heading_path_lower: cursor.heading_path.to_lowercase(),
            heading_level: cursor.heading_level,
            line_start,
            line_end,
            byte_start: cursor.byte_range.0,
            byte_end: cursor.byte_range.1,
        },
        section_text_lower: section_text.to_lowercase(),
        section_text,
        metadata: SectionMetadata {
            attributes,
            logbook,
        },
    })
}

/// Extract parser-owned Markdown sections from one document body.
#[must_use]
pub fn extract_sections(body: &str) -> Vec<MarkdownSection> {
    let structure = parse_markdown_structure(body);
    extract_sections_with_structure(body, &structure)
}

#[must_use]
pub(crate) fn extract_sections_with_structure(
    body: &str,
    structure: &MarkdownStructure,
) -> Vec<MarkdownSection> {
    let lines = body.lines().map(ToString::to_string).collect::<Vec<_>>();
    let headings = structure.headings().collect::<Vec<_>>();
    if headings.is_empty() {
        return vec![root_section(body)];
    }

    let total_lines = lines.len().max(1);
    let last_seen_byte = last_line_end_byte(body, &lines);
    let root = root_section_before_first_heading(&lines, &headings);
    let mut heading_stack = Vec::<String>::new();
    root.into_iter()
        .chain(headings.iter().enumerate().filter_map(|(index, heading)| {
            section_for_heading(
                &lines,
                &headings,
                &mut heading_stack,
                index,
                heading,
                total_lines,
                last_seen_byte,
            )
        }))
        .collect()
}

fn last_line_end_byte(body: &str, lines: &[String]) -> usize {
    lines
        .iter()
        .enumerate()
        .scan(0usize, |byte_offset, (index, line)| {
            let end = *byte_offset + line.len();
            *byte_offset = end + usize::from(index + 1 < lines.len());
            Some(end)
        })
        .last()
        .unwrap_or_else(|| usize::from(!body.is_empty()) * body.len())
}

fn root_section_before_first_heading(
    lines: &[String],
    headings: &[&crate::markdown_structure::MarkdownHeading],
) -> Option<MarkdownSection> {
    let first = headings.first()?;
    let cursor = SectionCursor {
        heading_title: "",
        heading_path: "",
        heading_level: 0,
        line_range: (1, first.start_line.saturating_sub(1).max(1)),
        byte_range: (0, first.byte_start.saturating_sub(1)),
    };
    let end = first.start_line.saturating_sub(1).min(lines.len());
    section_from_cursor(cursor, &lines[..end])
}

fn section_for_heading<'a>(
    lines: &'a [String],
    headings: &[&'a crate::markdown_structure::MarkdownHeading],
    heading_stack: &mut Vec<String>,
    index: usize,
    heading: &'a crate::markdown_structure::MarkdownHeading,
    total_lines: usize,
    last_seen_byte: usize,
) -> Option<MarkdownSection> {
    if heading_stack.len() >= heading.level {
        heading_stack.truncate(heading.level.saturating_sub(1));
    }
    heading_stack.push(heading.label.clone());
    let heading_path = heading_stack.join(" / ");

    let next_heading = headings.get(index + 1);
    let line_end = next_heading.map_or(total_lines.max(heading.start_line), |next| {
        next.start_line.saturating_sub(1).max(heading.start_line)
    });
    let byte_end = next_heading.map_or(last_seen_byte.max(heading.byte_start), |next| {
        next.byte_start.saturating_sub(1).max(heading.byte_start)
    });
    let body_lines = section_body_lines(lines, heading.end_line, line_end);

    section_from_cursor(
        SectionCursor {
            heading_title: heading.label.as_str(),
            heading_path: heading_path.as_str(),
            heading_level: heading.level,
            line_range: (heading.start_line, line_end),
            byte_range: (heading.byte_start, byte_end),
        },
        body_lines,
    )
}

fn section_body_lines(lines: &[String], heading_end_line: usize, line_end: usize) -> &[String] {
    let body_start_index = heading_end_line.saturating_add(1).saturating_sub(1);
    let body_end_exclusive = line_end.min(lines.len());
    if body_start_index >= body_end_exclusive {
        &[]
    } else {
        &lines[body_start_index..body_end_exclusive]
    }
}

fn root_section(body: &str) -> MarkdownSection {
    let section_text = body.trim().to_string();
    SectionCore {
        scope: SectionScope {
            heading_title: String::new(),
            heading_path: String::new(),
            heading_path_lower: String::new(),
            heading_level: 0,
            line_start: 1,
            line_end: body.lines().count().max(1),
            byte_start: 0,
            byte_end: body.len(),
        },
        section_text_lower: section_text.to_lowercase(),
        section_text,
        metadata: SectionMetadata {
            attributes: HashMap::new(),
            logbook: Vec::new(),
        },
    }
}
