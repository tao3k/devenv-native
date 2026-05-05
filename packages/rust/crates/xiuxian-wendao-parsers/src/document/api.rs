//! Public document parsing API for Markdown metadata envelopes.

use super::types::{DocumentCore, DocumentFormat, MarkdownDocument};
use crate::frontmatter::split_frontmatter;
use crate::markdown_structure::parse_markdown_document_metadata;
use serde_yaml::Value;

fn normalize_whitespace(raw: &str) -> String {
    raw.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn extract_tags(frontmatter: Option<&Value>) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let Some(value) = frontmatter else {
        return out;
    };
    let Some(tags_val) = value.get("tags") else {
        return out;
    };
    match tags_val {
        Value::String(s) => {
            let tag = s.trim();
            if !tag.is_empty() {
                out.push(tag.to_string());
            }
        }
        Value::Sequence(seq) => {
            for item in seq {
                if let Some(tag) = item.as_str() {
                    let cleaned = tag.trim();
                    if !cleaned.is_empty() {
                        out.push(cleaned.to_string());
                    }
                }
            }
        }
        _ => {}
    }
    out.sort();
    out.dedup();
    out
}

fn extract_title(
    frontmatter: Option<&Value>,
    body: &str,
    fallback_title: &str,
    structural_title: Option<&str>,
    allow_line_scan_fallback: bool,
) -> String {
    if let Some(value) = frontmatter {
        let frontmatter_title = value
            .get("title")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|s| !s.is_empty());
        if let Some(title) = frontmatter_title {
            return title.to_string();
        }
    }

    if let Some(title) = structural_title.map(str::trim).filter(|s| !s.is_empty()) {
        return title.to_string();
    }

    if allow_line_scan_fallback {
        for line in body.lines() {
            let trimmed = line.trim();
            if let Some(rest) = trimmed.strip_prefix("# ") {
                let candidate = rest.trim();
                if !candidate.is_empty() {
                    return candidate.to_string();
                }
            }
        }
    }
    fallback_title.to_string()
}

fn extract_lead(
    body: &str,
    structural_lead: Option<&str>,
    allow_line_scan_fallback: bool,
) -> String {
    if let Some(lead) = structural_lead
        .map(str::trim)
        .filter(|lead| !lead.is_empty())
    {
        return lead.chars().take(180).collect();
    }

    if !allow_line_scan_fallback {
        return String::new();
    }

    for line in body.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with("```") {
            continue;
        }
        let lead = normalize_whitespace(trimmed);
        if lead.is_empty() {
            continue;
        }
        return lead.chars().take(180).collect();
    }
    String::new()
}

fn extract_doc_type(frontmatter: Option<&Value>) -> Option<String> {
    let value = frontmatter?;
    value
        .get("type")
        .or_else(|| value.get("kind"))
        .and_then(Value::as_str)
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn count_words(body: &str) -> usize {
    body.split_whitespace().count()
}

pub(crate) fn parse_markdown_document_from_parts(
    frontmatter: Option<Value>,
    body: &str,
    fallback_title: &str,
    structural_title: Option<&str>,
    structural_lead: Option<&str>,
    allow_line_scan_fallback: bool,
) -> MarkdownDocument {
    let title = extract_title(
        frontmatter.as_ref(),
        body,
        fallback_title,
        structural_title,
        allow_line_scan_fallback,
    );
    let tags = extract_tags(frontmatter.as_ref());
    let doc_type = extract_doc_type(frontmatter.as_ref());
    let lead = extract_lead(body, structural_lead, allow_line_scan_fallback);
    let word_count = count_words(body);

    MarkdownDocument {
        raw_metadata: frontmatter,
        core: DocumentCore {
            format: DocumentFormat::Markdown,
            body: body.to_string(),
            title,
            tags,
            doc_type,
            lead,
            word_count,
        },
    }
}

/// Parse parser-owned Markdown document metadata from raw note content.
#[must_use]
pub fn parse_markdown_document(content: &str, fallback_title: &str) -> MarkdownDocument {
    let (frontmatter, body) = split_frontmatter(content);
    let metadata = parse_markdown_document_metadata(body);
    parse_markdown_document_from_parts(
        frontmatter,
        body,
        fallback_title,
        metadata.title(),
        metadata.lead_snippet(),
        false,
    )
}
