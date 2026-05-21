use comrak::{Options, nodes::AstNode};

use crate::studio::analysis::markdown::compile::types::EdgeDraft;
use crate::studio::types::AnalysisEdge;

pub(crate) fn markdown_code_semantic_type(label: &str) -> String {
    let normalized = label
        .strip_prefix("code block (")
        .and_then(|value| value.strip_suffix(')'))
        .map(str::trim)
        .map(str::to_ascii_lowercase);

    match normalized.as_deref() {
        Some("mermaid") => "mermaid".to_string(),
        Some(language) if !language.is_empty() => format!("code:{language}"),
        _ => "code".to_string(),
    }
}

pub(crate) fn markdown_options() -> Options<'static> {
    let mut options = Options::default();
    options.extension.tasklist = true;
    options.extension.table = true;
    options.extension.math_dollars = true;
    options
}

pub(crate) fn line_start<'a>(node: &'a AstNode<'a>) -> usize {
    source_line(node.data().sourcepos.start.line)
}

pub(crate) fn line_end<'a>(node: &'a AstNode<'a>) -> usize {
    source_line(node.data().sourcepos.end.line).max(line_start(node))
}

pub(crate) fn source_line(raw: usize) -> usize {
    raw.max(1)
}

pub(crate) fn current_section_context(section_stack: &[(usize, String)]) -> &str {
    section_stack
        .last()
        .map_or("doc:0", |(_, node_id)| node_id.as_str())
}

pub(crate) fn make_edge(edge_seq: usize, draft: EdgeDraft<'_>) -> AnalysisEdge {
    AnalysisEdge {
        id: format!("edge:{edge_seq}").into(),
        kind: draft.kind,
        source_id: draft.source_id.into(),
        target_id: draft.target_id.into(),
        label: draft.label,
        evidence: crate::studio::types::AnalysisEvidence {
            path: draft.path.to_string(),
            line_start: draft.line_start,
            line_end: draft.line_end,
            confidence: draft.confidence,
        },
    }
}

pub(crate) fn normalize_reference(target: &str) -> String {
    target.trim().trim_matches('#').to_string()
}

pub(crate) fn slugify(input: &str) -> String {
    let mut slug = String::with_capacity(input.len());
    let mut prev_dash = false;
    for ch in input.chars().flat_map(char::to_lowercase) {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch);
            prev_dash = false;
        } else if !prev_dash {
            slug.push('-');
            prev_dash = true;
        }
    }
    slug.trim_matches('-').to_string()
}

pub(crate) fn slice_content_lines(content: &str, line_start: usize, line_end: usize) -> String {
    let start = line_start.max(1);
    let end = line_end.max(start);
    content
        .lines()
        .enumerate()
        .filter_map(|(index, line)| {
            let line_no = index + 1;
            (line_no >= start && line_no <= end).then_some(line)
        })
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

pub(crate) fn build_stable_fingerprint(value: &str) -> String {
    let mut hash = 5_381_u32;

    for byte in value.bytes() {
        hash = ((hash << 5).wrapping_add(hash)) ^ u32::from(byte);
    }

    format!("fp:{hash:08x}")
}

pub(crate) fn estimate_token_count(value: &str) -> usize {
    let normalized = value.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.is_empty() {
        0
    } else {
        normalized.len().div_ceil(4)
    }
}

pub(crate) fn node_key<'a>(node: &'a AstNode<'a>) -> usize {
    std::ptr::from_ref(node) as usize
}
