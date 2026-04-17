use super::fences::parse_fence_marker;
use super::types::{MarkdownSyntaxLintCode, MarkdownSyntaxLintIssue};
use crate::markdown_structure::parse_markdown_structure;
use crate::targets::MarkdownTargetOccurrenceKind;
use regex::Regex;
use std::sync::LazyLock;

static MIXED_WIKILINK_MARKDOWN_LINK_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"!?\[\[[^\]\n]+\]\]\([^)\n]*\)")
        .expect("hardcoded mixed wikilink regex should compile")
});

pub(super) fn lint_obsidian_wikilinks(
    body: &str,
    line_offset: usize,
    issues: &mut Vec<MarkdownSyntaxLintIssue>,
) {
    let blocked_spans = lint_mixed_wikilink_markdown_links(body, line_offset, issues);

    let structure = parse_markdown_structure(body);
    for occurrence in structure
        .targets()
        .iter()
        .filter(|target| target.kind == MarkdownTargetOccurrenceKind::WikiLink)
    {
        if blocked_spans.iter().any(|(start, end)| {
            occurrence.byte_range.0 >= *start && occurrence.byte_range.1 <= *end
        }) {
            continue;
        }
        let Some(literal) = body.get(occurrence.byte_range.0..occurrence.byte_range.1) else {
            continue;
        };
        let Some(parts) = parse_wikilink_literal_parts(literal) else {
            continue;
        };
        let line = line_offset + occurrence.line_range.0.saturating_sub(1);
        let column = byte_offset_to_column(body, occurrence.byte_range.0);

        let Some(label) = parts.label else {
            issues.push(MarkdownSyntaxLintIssue {
                code: MarkdownSyntaxLintCode::BareObsidianWikilink,
                message: format!(
                    "bare wikilink `{literal}` is not allowed in repository Markdown; use `[[target|label]]` or `[label](target)`"
                ),
                line,
                column,
            });
            continue;
        };

        if is_redundant_obsidian_label(occurrence.target.as_str(), label) {
            issues.push(MarkdownSyntaxLintIssue {
                code: MarkdownSyntaxLintCode::RedundantObsidianLabel,
                message: format!(
                    "wikilink `{literal}` repeats the target or addressed heading as the display label; use a human-readable label or a Markdown link if the raw path must stay visible"
                ),
                line,
                column,
            });
            continue;
        }

        if looks_like_reversed_obsidian_alias(parts.target, label) {
            issues.push(MarkdownSyntaxLintIssue {
                code: MarkdownSyntaxLintCode::NonCanonicalObsidianAliasOrder,
                message: format!(
                    "Obsidian alias wikilinks use `[[target|label]]`; `{}` looks reversed because the right-hand side `{}` looks like a repository target path or address",
                    literal, label
                ),
                line,
                column,
            });
        }
    }
}

#[derive(Clone, Copy)]
struct WikilinkLiteralParts<'a> {
    target: &'a str,
    label: Option<&'a str>,
}

#[derive(Clone, Copy, Debug)]
struct FenceStart {
    marker: char,
    width: usize,
}

fn lint_mixed_wikilink_markdown_links(
    body: &str,
    line_offset: usize,
    issues: &mut Vec<MarkdownSyntaxLintIssue>,
) -> Vec<(usize, usize)> {
    let mut open_fence: Option<FenceStart> = None;
    let mut blocked_spans = Vec::new();
    let mut line_byte_start = 0usize;

    for (index, raw_line) in body.lines().enumerate() {
        let line_number = line_offset + index;
        let line = raw_line.trim_end_matches('\r');
        let Some(candidate) = parse_fence_marker(line) else {
            if open_fence.is_none() {
                issues.extend(collect_mixed_line_issues(
                    line,
                    line_number,
                    line_byte_start,
                    &mut blocked_spans,
                ));
            }
            line_byte_start += raw_line.len();
            if body.as_bytes().get(line_byte_start) == Some(&b'\n') {
                line_byte_start += 1;
            }
            continue;
        };
        match open_fence {
            Some(start)
                if candidate.marker == start.marker
                    && candidate.width >= start.width
                    && candidate.trailing.trim().is_empty() =>
            {
                open_fence = None;
            }
            None => {
                open_fence = Some(FenceStart {
                    marker: candidate.marker,
                    width: candidate.width,
                });
            }
            Some(_) => {}
        }
        line_byte_start += raw_line.len();
        if body.as_bytes().get(line_byte_start) == Some(&b'\n') {
            line_byte_start += 1;
        }
    }

    blocked_spans
}

fn collect_mixed_line_issues(
    line: &str,
    line_number: usize,
    line_byte_start: usize,
    blocked_spans: &mut Vec<(usize, usize)>,
) -> Vec<MarkdownSyntaxLintIssue> {
    let mut issues = Vec::new();
    let mut segment_start = 0usize;
    let mut inline_code_width: Option<usize> = None;
    let bytes = line.as_bytes();
    let mut cursor = 0usize;

    while cursor < bytes.len() {
        if bytes[cursor] != b'`' {
            cursor += line[cursor..]
                .chars()
                .next()
                .map_or(1, std::primitive::char::len_utf8);
            continue;
        }

        let tick_start = cursor;
        while cursor < bytes.len() && bytes[cursor] == b'`' {
            cursor += 1;
        }
        let tick_width = cursor - tick_start;

        match inline_code_width {
            Some(active_width) if active_width == tick_width => {
                inline_code_width = None;
                segment_start = cursor;
            }
            Some(_) => {}
            None => {
                issues.extend(collect_mixed_segment_issues(
                    line,
                    line_number,
                    &line[segment_start..tick_start],
                    segment_start,
                    line_byte_start,
                    blocked_spans,
                ));
            }
        }

        if inline_code_width.is_none() {
            inline_code_width = Some(tick_width);
        }
    }

    if inline_code_width.is_none() && segment_start < line.len() {
        issues.extend(collect_mixed_segment_issues(
            line,
            line_number,
            &line[segment_start..],
            segment_start,
            line_byte_start,
            blocked_spans,
        ));
    }

    issues
}

fn collect_mixed_segment_issues(
    line: &str,
    line_number: usize,
    segment: &str,
    base_offset: usize,
    line_byte_start: usize,
    blocked_spans: &mut Vec<(usize, usize)>,
) -> Vec<MarkdownSyntaxLintIssue> {
    let mut issues = Vec::new();
    for matched in MIXED_WIKILINK_MARKDOWN_LINK_REGEX.find_iter(segment) {
        let absolute_start = base_offset + matched.start();
        if is_escaped(line, absolute_start) {
            continue;
        }
        blocked_spans.push((
            line_byte_start + absolute_start,
            line_byte_start + base_offset + matched.end(),
        ));
        let column = line[..absolute_start].chars().count() + 1;
        issues.push(MarkdownSyntaxLintIssue {
            code: MarkdownSyntaxLintCode::MixedWikilinkMarkdownLink,
            message:
                "mixed wikilink/Markdown link syntax is invalid; use `[[target|label]]` or `[label](target)`"
                    .to_string(),
            line: line_number,
            column,
        });
    }
    issues
}

fn parse_wikilink_literal_parts(literal: &str) -> Option<WikilinkLiteralParts<'_>> {
    let inner = literal
        .strip_prefix("[[")
        .and_then(|value| value.strip_suffix("]]"))?;
    match inner.split_once('|') {
        Some((target, label)) => Some(WikilinkLiteralParts {
            target: target.trim(),
            label: Some(label.trim()),
        }),
        None => Some(WikilinkLiteralParts {
            target: inner.trim(),
            label: None,
        }),
    }
}

fn looks_like_reversed_obsidian_alias(left: &str, right: &str) -> bool {
    looks_like_obsidian_target_hint(right) && !looks_like_obsidian_target_hint(left)
}

fn is_redundant_obsidian_label(target: &str, label: &str) -> bool {
    let target = target.trim();
    let label = label.trim();
    target == label
        || target.split_once('#').is_some_and(|(_path, fragment)| {
            normalize_label_hint(fragment) == normalize_label_hint(label)
        })
}

fn looks_like_obsidian_target_hint(value: &str) -> bool {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return false;
    }

    trimmed.starts_with("./")
        || trimmed.starts_with("../")
        || trimmed.starts_with('/')
        || trimmed.starts_with('#')
        || trimmed.starts_with("id:")
        || trimmed.starts_with("obsidian://")
        || trimmed.starts_with("wendao://")
        || trimmed.contains('/')
        || trimmed.contains('#')
        || trimmed.ends_with(".md")
        || trimmed.ends_with(".markdown")
}

fn normalize_label_hint(value: &str) -> String {
    let mut normalized = String::new();
    let mut previous_was_space = false;
    for character in value.trim().trim_start_matches('^').chars() {
        let mapped = match character {
            '-' | '_' | '/' => ' ',
            other => other.to_ascii_lowercase(),
        };
        if mapped.is_whitespace() {
            if !previous_was_space && !normalized.is_empty() {
                normalized.push(' ');
            }
            previous_was_space = true;
            continue;
        }
        normalized.push(mapped);
        previous_was_space = false;
    }
    normalized
}

fn byte_offset_to_column(text: &str, byte_offset: usize) -> usize {
    let safe_offset = byte_offset.min(text.len());
    let line_start = text[..safe_offset].rfind('\n').map_or(0, |index| index + 1);
    text[line_start..safe_offset].chars().count() + 1
}

fn is_escaped(line: &str, start: usize) -> bool {
    let preceding = &line[..start];
    let slash_count = preceding
        .chars()
        .rev()
        .take_while(|character| *character == '\\')
        .count();
    slash_count % 2 == 1
}
