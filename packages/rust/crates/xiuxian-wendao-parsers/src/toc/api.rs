use super::types::{
    MarkdownOutlineDocument, MarkdownOutlineHeading, MarkdownTocDocument, TocDocument,
};
use crate::document::parse_markdown_document_from_parts;
use crate::frontmatter::split_frontmatter;
use crate::markdown_structure::parse_markdown_structure;
use crate::section_create::parse_heading_line;
use crate::sections::extract_sections_with_structure;
use serde_yaml::Value;

/// Parse one parser-owned Markdown TOC surface from raw content.
#[must_use]
pub fn parse_markdown_toc(content: &str, fallback_title: &str) -> MarkdownTocDocument {
    let (frontmatter, body) = split_frontmatter(content);
    let structure = parse_markdown_structure(body);
    let document = parse_markdown_document_from_parts(
        frontmatter,
        body,
        fallback_title,
        structure.first_heading_title(),
        structure.lead_snippet(),
        false,
    );
    let sections = extract_sections_with_structure(body, &structure);
    TocDocument { document, sections }
}

/// Parse one lightweight parser-owned Markdown outline from raw content.
#[must_use]
pub fn parse_markdown_outline(content: &str, fallback_title: &str) -> MarkdownOutlineDocument {
    let (frontmatter, body) = split_frontmatter(content);
    let headings = scan_outline_headings(body);
    let title = extract_outline_title(frontmatter.as_ref(), headings.first(), fallback_title);
    let doc_type = extract_outline_doc_type(frontmatter.as_ref());

    MarkdownOutlineDocument {
        title,
        doc_type,
        line_count: body.lines().count().max(1),
        headings,
    }
}

fn extract_outline_title(
    frontmatter: Option<&Value>,
    first_heading: Option<&MarkdownOutlineHeading>,
    fallback_title: &str,
) -> String {
    if let Some(title) = frontmatter
        .and_then(|value| value.get("title"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|title| !title.is_empty())
    {
        return title.to_string();
    }

    if let Some(title) = first_heading
        .map(|heading| heading.title.as_str())
        .map(str::trim)
        .filter(|title| !title.is_empty())
    {
        return title.to_string();
    }

    fallback_title.to_string()
}

fn extract_outline_doc_type(frontmatter: Option<&Value>) -> Option<String> {
    frontmatter
        .and_then(|value| value.get("type").or_else(|| value.get("kind")))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

fn scan_outline_headings(body: &str) -> Vec<MarkdownOutlineHeading> {
    let mut headings = Vec::<(usize, usize, String)>::new();
    let mut open_fence = None::<(char, usize)>;

    for (line_index, line) in body.lines().enumerate() {
        let trimmed = line.trim_start();
        if let Some((marker, count)) = parse_fence_delimiter(trimmed) {
            match open_fence {
                Some((open_marker, open_count)) if open_marker == marker && count >= open_count => {
                    open_fence = None;
                    continue;
                }
                None => {
                    open_fence = Some((marker, count));
                    continue;
                }
                Some(_) => continue,
            }
        }

        if open_fence.is_some() {
            continue;
        }

        if let Some((level, title)) = parse_heading_line(trimmed) {
            headings.push((line_index + 1, level, title));
        }
    }

    let total_lines = body.lines().count().max(1);
    headings
        .iter()
        .enumerate()
        .map(
            |(index, (start_line, level, title))| MarkdownOutlineHeading {
                title: title.clone(),
                level: *level,
                line_range: (
                    *start_line,
                    headings.get(index + 1).map_or(
                        total_lines.max(*start_line),
                        |(next_start_line, _, _)| {
                            next_start_line.saturating_sub(1).max(*start_line)
                        },
                    ),
                ),
            },
        )
        .collect()
}

fn parse_fence_delimiter(line: &str) -> Option<(char, usize)> {
    let mut chars = line.chars();
    let marker = chars.next()?;
    if marker != '`' && marker != '~' {
        return None;
    }
    let count = line
        .chars()
        .take_while(|character| *character == marker)
        .count();
    if count < 3 {
        return None;
    }
    Some((marker, count))
}
