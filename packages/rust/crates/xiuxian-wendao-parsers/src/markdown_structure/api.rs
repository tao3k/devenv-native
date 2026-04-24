use comrak::{
    Arena, Options,
    nodes::{AstNode, NodeValue},
    parse_document,
};

use crate::AddressedTarget;
use crate::references::{MarkdownReference, MarkdownReferenceKind};
use crate::sourcepos::sourcepos_to_byte_range;
use crate::targets::{MarkdownTargetOccurrence, MarkdownTargetOccurrenceKind};

use super::target_scan::extend_loose_markdown_targets;
use super::types::{
    MarkdownDocumentMetadata, MarkdownHeading, MarkdownStructuralItem, MarkdownStructure,
    MarkdownTask,
};

#[derive(Debug, Clone, Copy)]
struct MarkdownOccurrenceSpan {
    byte_range: Option<(usize, usize)>,
    line_range: (usize, usize),
}

#[must_use]
pub(crate) fn parse_markdown_structure(body: &str) -> MarkdownStructure {
    let arena = Arena::new();
    let root = parse_markdown_root(body, &arena);
    let mut items = Vec::new();
    let mut lead = None;
    let mut references = Vec::new();
    let mut targets = Vec::new();

    for node in root.descendants() {
        let sourcepos = node.data().sourcepos;
        let span = MarkdownOccurrenceSpan {
            byte_range: sourcepos_to_byte_range(body, sourcepos),
            line_range: (sourcepos.start.line.max(1), sourcepos.end.line.max(1)),
        };

        match &node.data().value {
            NodeValue::Paragraph if lead.is_none() => {
                lead = extract_raw_paragraph_snippet(body, span.byte_range);
            }
            NodeValue::Heading(heading) => push_heading(
                &mut items,
                node,
                heading.level as usize,
                span.byte_range,
                sourcepos.start.line.max(1),
                sourcepos.end.line.max(1),
            ),
            NodeValue::TaskItem(_) => {
                let label = collect_plain_text(node);
                if label.is_empty() {
                    continue;
                }
                items.push(MarkdownStructuralItem::Task(MarkdownTask { label }));
            }
            NodeValue::Link(link) => {
                push_markdown_link(&mut references, &mut targets, body, link.url.as_str(), span);
            }
            NodeValue::Image(image) => push_target(
                &mut targets,
                body,
                MarkdownTargetOccurrenceKind::MarkdownImage,
                image.url.clone(),
                span,
            ),
            NodeValue::WikiLink(link) => push_wikilink(
                &mut references,
                &mut targets,
                node,
                body,
                link.url.as_str(),
                span,
            ),
            _ => {}
        }
    }
    extend_embedded_wikilink_targets(&mut targets, body);
    extend_loose_markdown_targets(&mut targets, body);
    targets.sort_by_key(|target| target.byte_range.0);

    MarkdownStructure {
        items,
        lead,
        references,
        targets,
    }
}

#[must_use]
pub(crate) fn parse_markdown_document_metadata(body: &str) -> MarkdownDocumentMetadata {
    let arena = Arena::new();
    let root = parse_markdown_root(body, &arena);
    let mut title = None;
    let mut lead = None;

    for node in root.descendants() {
        let sourcepos = node.data().sourcepos;
        let byte_range = sourcepos_to_byte_range(body, sourcepos);

        match &node.data().value {
            NodeValue::Paragraph if lead.is_none() => {
                lead = extract_raw_paragraph_snippet(body, byte_range);
                if title.is_some() && lead.is_some() {
                    break;
                }
            }
            NodeValue::Heading(_) if title.is_none() => {
                let label = collect_plain_text(node);
                if !label.is_empty() {
                    title = Some(label);
                    if lead.is_some() {
                        break;
                    }
                }
            }
            _ => {}
        }
    }

    MarkdownDocumentMetadata { title, lead }
}

fn parse_markdown_root<'a>(body: &'a str, arena: &'a Arena<'a>) -> &'a AstNode<'a> {
    let mut options = Options::default();
    options.extension.wikilinks_title_after_pipe = true;
    parse_document(arena, body, &options)
}

fn push_heading<'a>(
    items: &mut Vec<MarkdownStructuralItem>,
    node: &'a AstNode<'a>,
    level: usize,
    byte_range: Option<(usize, usize)>,
    start_line: usize,
    end_line: usize,
) {
    let label = collect_plain_text(node);
    if label.is_empty() {
        return;
    }
    let Some((byte_start, _)) = byte_range else {
        return;
    };
    items.push(MarkdownStructuralItem::Heading(MarkdownHeading {
        label,
        level,
        start_line,
        end_line: end_line.max(start_line),
        byte_start,
    }));
}

fn push_markdown_link(
    references: &mut Vec<MarkdownReference>,
    targets: &mut Vec<MarkdownTargetOccurrence>,
    body: &str,
    raw_target: &str,
    span: MarkdownOccurrenceSpan,
) {
    if let Some(reference) = parse_reference(
        MarkdownReferenceKind::Markdown,
        raw_target,
        span.byte_range,
        body,
    ) {
        references.push(reference);
    }
    push_target(
        targets,
        body,
        MarkdownTargetOccurrenceKind::MarkdownLink,
        raw_target.to_string(),
        span,
    );
}

fn push_wikilink(
    references: &mut Vec<MarkdownReference>,
    targets: &mut Vec<MarkdownTargetOccurrence>,
    node: &AstNode<'_>,
    body: &str,
    raw_target: &str,
    span: MarkdownOccurrenceSpan,
) {
    let is_embed = is_embedded_wikilink(node);
    if !is_embed
        && let Some(reference) = parse_reference(
            MarkdownReferenceKind::WikiLink,
            raw_target,
            span.byte_range,
            body,
        )
    {
        references.push(reference);
    }
    push_target(
        targets,
        body,
        if is_embed {
            MarkdownTargetOccurrenceKind::WikiEmbed
        } else {
            MarkdownTargetOccurrenceKind::WikiLink
        },
        raw_target.to_string(),
        span,
    );
}

fn push_target(
    targets: &mut Vec<MarkdownTargetOccurrence>,
    body: &str,
    kind: MarkdownTargetOccurrenceKind,
    target: String,
    span: MarkdownOccurrenceSpan,
) {
    if let Some((start, end)) = span.byte_range {
        targets.push(MarkdownTargetOccurrence::new(
            kind,
            target,
            target_surface(body, start, end),
            (start, end),
            span.line_range,
        ));
    }
}

fn extend_embedded_wikilink_targets(targets: &mut Vec<MarkdownTargetOccurrence>, body: &str) {
    let mut offset = 0;
    while let Some(relative_start) = body[offset..].find("![[") {
        let start = offset + relative_start;
        let search_from = start + 3;
        let Some(relative_end) = body[search_from..].find("]]") else {
            break;
        };
        let end = search_from + relative_end + 2;
        if targets.iter().any(|target| {
            target.kind == MarkdownTargetOccurrenceKind::WikiEmbed
                && target.byte_range == (start, end)
        }) {
            offset = end;
            continue;
        }

        let surface = body[start..end].to_string();
        let target = parse_embedded_wikilink_target(surface.as_str());
        let line_range = line_range_for_span(body, start, end);
        targets.push(MarkdownTargetOccurrence::new(
            MarkdownTargetOccurrenceKind::WikiEmbed,
            target,
            surface,
            (start, end),
            line_range,
        ));
        offset = end;
    }
}

fn parse_embedded_wikilink_target(surface: &str) -> String {
    let inner = surface
        .trim()
        .strip_prefix("![[")
        .and_then(|value| value.strip_suffix("]]"))
        .unwrap_or_default()
        .trim();
    inner
        .split_once('|')
        .map_or(inner, |(target, _)| target)
        .trim()
        .to_string()
}

fn line_range_for_span(body: &str, start: usize, end: usize) -> (usize, usize) {
    let start_line = body[..start].bytes().filter(|byte| *byte == b'\n').count() + 1;
    let span = &body[start..end];
    let end_line = start_line + span.bytes().filter(|byte| *byte == b'\n').count();
    (start_line, end_line.max(start_line))
}

fn target_surface(body: &str, start: usize, end: usize) -> String {
    body.get(start..end).unwrap_or_default().to_string()
}

fn collect_plain_text<'a>(node: &'a AstNode<'a>) -> String {
    let mut output = String::new();

    for descendant in node.descendants() {
        match &descendant.data().value {
            NodeValue::Text(text) => push_segment(&mut output, text),
            NodeValue::Code(code) => push_segment(&mut output, code.literal.as_str()),
            NodeValue::Math(math) => push_segment(&mut output, math.literal.as_str()),
            NodeValue::WikiLink(link) => push_segment(&mut output, link.url.as_str()),
            NodeValue::LineBreak | NodeValue::SoftBreak => push_segment(&mut output, " "),
            _ => {}
        }
    }

    normalize_whitespace(output.as_str())
}

fn push_segment(output: &mut String, segment: &str) {
    if segment.is_empty() {
        return;
    }
    if !output.is_empty() {
        output.push(' ');
    }
    output.push_str(segment);
}

fn normalize_whitespace(raw: &str) -> String {
    raw.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn extract_raw_paragraph_snippet(body: &str, byte_range: Option<(usize, usize)>) -> Option<String> {
    let (start, end) = byte_range?;
    let raw = body.get(start..end)?.trim();
    if raw.is_empty() {
        return None;
    }
    let normalized = normalize_whitespace(raw);
    if normalized.is_empty() {
        return None;
    }
    Some(normalized.chars().take(180).collect())
}

fn parse_reference(
    kind: MarkdownReferenceKind,
    raw_target: &str,
    byte_range: Option<(usize, usize)>,
    body: &str,
) -> Option<MarkdownReference> {
    let addressed_target = parse_reference_target(raw_target)?;
    let (start, end) = byte_range?;
    let original = body.get(start..end)?.to_string();

    Some(MarkdownReference::new(kind, addressed_target, original))
}

fn parse_reference_target(raw: &str) -> Option<AddressedTarget> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }

    if trimmed.starts_with('#') {
        return Some(AddressedTarget::new(None, Some(trimmed.to_string())));
    }

    let Some((target, raw_address)) = trimmed.split_once('#') else {
        return Some(AddressedTarget::new(Some(trimmed.to_string()), None));
    };

    let target = target.trim();
    if target.is_empty() {
        return None;
    }

    let raw_address = raw_address.trim();
    let target_address = if raw_address.is_empty() {
        None
    } else {
        Some(format!("#{raw_address}"))
    };

    Some(AddressedTarget::new(
        Some(target.to_string()),
        target_address,
    ))
}

fn is_embedded_wikilink(node: &AstNode<'_>) -> bool {
    let Some(previous) = node.previous_sibling() else {
        return false;
    };
    let NodeValue::Text(text) = &previous.data().value else {
        return false;
    };
    text.as_ref().ends_with('!')
}
