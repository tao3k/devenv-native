//! Markdown section insertion-point analysis.

use super::types::{InsertionInfo, SiblingInfo};

type HeadingPosition = (usize, usize, String);

/// Find the insertion point for one missing Markdown heading path.
///
/// The result captures the byte offset where new content should be inserted,
/// the heading level to start from, the remaining headings that still need to
/// be created, and bounded sibling context for response rendering.
#[must_use]
pub fn find_insertion_point(doc_content: &str, path_components: &[String]) -> InsertionInfo {
    if doc_content.trim().is_empty() {
        return InsertionInfo {
            insertion_byte: 0,
            start_level: 1,
            remaining_path: path_components.to_vec(),
            prev_sibling: None,
            next_sibling: None,
        };
    }

    if path_components.is_empty() {
        return InsertionInfo {
            insertion_byte: doc_content.len(),
            start_level: 1,
            remaining_path: Vec::new(),
            prev_sibling: None,
            next_sibling: None,
        };
    }

    let lines: Vec<&str> = doc_content.lines().collect();
    let heading_positions = parse_headings(&lines);
    let (matched_depth, last_matched_level, last_matched_end_line, matched_line_idx) =
        find_deepest_match_with_position(&heading_positions, path_components);
    let insertion_byte = calculate_insertion_byte(&lines, matched_depth, last_matched_end_line);
    let remaining_path = path_components[matched_depth..].to_vec();
    let start_level = if matched_depth == 0 {
        1
    } else {
        last_matched_level + 1
    };
    let (prev_sibling, next_sibling) = find_sibling_context(
        &heading_positions,
        &lines,
        matched_depth,
        matched_line_idx,
        start_level,
    );

    InsertionInfo {
        insertion_byte,
        start_level,
        remaining_path,
        prev_sibling,
        next_sibling,
    }
}

/// Parse one Markdown ATX heading line into its level plus normalized title.
#[must_use]
pub fn parse_heading_line(line: &str) -> Option<(usize, String)> {
    if !line.starts_with('#') {
        return None;
    }

    let mut level = 0;
    for character in line.chars() {
        if character == '#' {
            level += 1;
        } else {
            break;
        }
    }

    if level == 0 || level > 6 {
        return None;
    }

    let title = line[level..].trim().to_string();
    if title.is_empty() {
        return None;
    }

    Some((level, title))
}

fn parse_headings(lines: &[&str]) -> Vec<HeadingPosition> {
    let mut headings = Vec::new();

    for (line_idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim_start();
        if let Some((level, title)) = parse_heading_line(trimmed) {
            headings.push((line_idx, level, title));
        }
    }

    headings
}

fn find_deepest_match_with_position(
    heading_positions: &[HeadingPosition],
    path_components: &[String],
) -> (usize, usize, usize, Option<usize>) {
    let mut matched_depth = 0;
    let mut last_matched_level = 0;
    let mut last_matched_end_line = 0;
    let mut matched_line_idx = None;

    for (depth, target_title) in path_components.iter().enumerate() {
        let expected_level = depth + 1;
        let mut found = false;

        for &(line_idx, level, ref title) in heading_positions {
            if title == target_title && level == expected_level {
                matched_depth = depth + 1;
                last_matched_level = level;
                last_matched_end_line = find_section_end(heading_positions, line_idx, level);
                matched_line_idx = Some(line_idx);
                found = true;
                break;
            }
        }

        if !found {
            break;
        }
    }

    (
        matched_depth,
        last_matched_level,
        last_matched_end_line,
        matched_line_idx,
    )
}

fn find_sibling_context(
    heading_positions: &[HeadingPosition],
    lines: &[&str],
    matched_depth: usize,
    matched_line_idx: Option<usize>,
    target_level: usize,
) -> (Option<SiblingInfo>, Option<SiblingInfo>) {
    if heading_positions.is_empty() {
        return (None, None);
    }

    let mut prev_sibling = None;
    let next_sibling = None;
    let parent_line = matched_line_idx.unwrap_or(0);
    let parent_level = if matched_depth > 0 { matched_depth } else { 0 };
    let end_boundary = if matched_depth > 0 {
        heading_positions
            .iter()
            .find(|&&(line_idx, level, _)| line_idx > parent_line && level <= parent_level)
            .map_or(usize::MAX, |&(line_idx, _, _)| line_idx)
    } else {
        usize::MAX
    };

    let siblings_at_level: Vec<_> = heading_positions
        .iter()
        .filter(|&&(line_idx, level, _)| {
            level == target_level && line_idx > parent_line && line_idx < end_boundary
        })
        .collect();

    if let Some(last) = siblings_at_level.last() {
        let preview = extract_preview(lines, last.0);
        prev_sibling = Some(SiblingInfo {
            title: last.2.clone(),
            preview,
        });
    }

    (prev_sibling, next_sibling)
}

fn extract_preview(lines: &[&str], heading_line_idx: usize) -> String {
    for line in lines.iter().skip(heading_line_idx + 1).take(3) {
        let trimmed = line.trim();
        if !trimmed.is_empty() && !trimmed.starts_with('#') && !trimmed.starts_with(':') {
            return trimmed.chars().take(80).collect();
        }
    }
    String::new()
}

fn find_section_end(
    heading_positions: &[HeadingPosition],
    start_line_idx: usize,
    section_level: usize,
) -> usize {
    for &(line_idx, level, _) in heading_positions {
        if line_idx > start_line_idx && level <= section_level {
            return line_idx;
        }
    }
    usize::MAX
}

fn calculate_insertion_byte(
    lines: &[&str],
    matched_depth: usize,
    last_matched_end_line: usize,
) -> usize {
    if matched_depth == 0 {
        return lines.iter().map(|line| line.len() + 1).sum();
    }

    if last_matched_end_line == usize::MAX {
        return lines.iter().map(|line| line.len() + 1).sum();
    }

    lines
        .iter()
        .take(last_matched_end_line)
        .map(|line| line.len() + 1)
        .sum()
}
