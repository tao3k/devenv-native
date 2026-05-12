//! Org document metadata extraction.

use std::collections::BTreeMap;

use orgize::Org;
use orgize::rowan::ast::AstNode;
use orgize::syntax_ast::Headline;

use crate::document::{
    DocumentCore, DocumentFormat, DocumentType, OrgDocument, OrgDocumentMetadata,
};

const LEAD_LIMIT: usize = 180;

/// Parse parser-owned Org document metadata from raw note content.
#[must_use]
pub fn parse_org_document(content: &str, fallback_title: &str) -> OrgDocument {
    let org = Org::parse(content);
    let raw_metadata = extract_org_metadata(&org);
    let body = strip_leading_org_metadata(content);
    let body_org = Org::parse(body.as_str());
    let first_heading = first_headline_title(&body_org);
    let title = org
        .title()
        .map(|title| title.trim().to_string())
        .filter(|title| !title.is_empty())
        .or(first_heading)
        .unwrap_or_else(|| fallback_title.to_string());
    let tags = extract_document_tags(&raw_metadata);
    let doc_type = extract_doc_type(&raw_metadata);
    let lead = extract_lead(body.as_str());
    let word_count = body.split_whitespace().count();

    OrgDocument {
        raw_metadata: Some(raw_metadata),
        core: DocumentCore {
            format: DocumentFormat::Org,
            body,
            title,
            tags,
            doc_type,
            lead,
            word_count,
        },
    }
}

fn extract_org_metadata(org: &Org) -> OrgDocumentMetadata {
    let mut keywords = BTreeMap::<String, Vec<String>>::new();
    for keyword in org.keywords() {
        let key = keyword.key().trim().to_uppercase();
        let value = keyword.value().trim().to_string();
        if key.is_empty() || value.is_empty() {
            continue;
        }
        keywords.entry(key).or_default().push(value);
    }

    let properties = org
        .syntax_document()
        .properties()
        .map(|drawer| {
            drawer
                .iter()
                .filter_map(|(key, value)| normalize_property_pair(key.as_ref(), value.as_ref()))
                .collect()
        })
        .unwrap_or_default();

    OrgDocumentMetadata {
        keywords,
        properties,
    }
}

fn normalize_property_pair(key: &str, value: &str) -> Option<(String, String)> {
    let key = key.trim().to_uppercase();
    let value = value.trim().to_string();
    if key.is_empty() || value.is_empty() {
        None
    } else {
        Some((key, value))
    }
}

fn first_headline_title(org: &Org) -> Option<String> {
    org.syntax_document()
        .syntax()
        .descendants()
        .find_map(Headline::cast)
        .map(|headline| headline.title_raw().trim().to_string())
        .filter(|title| !title.is_empty())
}

fn strip_leading_org_metadata(content: &str) -> String {
    let mut offset = 0usize;
    loop {
        let Some(line) = line_at_offset(content, offset) else {
            return String::new();
        };
        let line_len = source_line_len(content, offset, line);
        let trimmed = line.trim();

        if trimmed.is_empty() || is_org_keyword(trimmed) {
            offset += line_len;
            continue;
        }

        if trimmed.eq_ignore_ascii_case(":PROPERTIES:") {
            offset = skip_property_drawer(content, offset + line_len);
            continue;
        }

        break;
    }

    content[offset..].to_string()
}

fn line_at_offset(content: &str, offset: usize) -> Option<&str> {
    content[offset..].lines().next()
}

fn source_line_len(content: &str, offset: usize, line: &str) -> usize {
    line.len() + line_ending_len(content, offset, line.len())
}

fn skip_property_drawer(content: &str, mut offset: usize) -> usize {
    while offset < content.len() {
        let Some(property_line) = line_at_offset(content, offset) else {
            return content.len();
        };
        let property_line_len = source_line_len(content, offset, property_line);
        offset += property_line_len;
        if property_line.trim().eq_ignore_ascii_case(":END:") {
            break;
        }
    }
    offset
}

fn line_ending_len(content: &str, line_offset: usize, line_len: usize) -> usize {
    let after_line = line_offset + line_len;
    if content[after_line..].starts_with("\r\n") {
        2
    } else {
        usize::from(content[after_line..].starts_with('\n'))
    }
}

fn is_org_keyword(trimmed: &str) -> bool {
    trimmed.starts_with("#+") && trimmed.contains(':')
}

fn extract_document_tags(metadata: &OrgDocumentMetadata) -> Vec<String> {
    let mut tags = metadata
        .keywords
        .get("FILETAGS")
        .into_iter()
        .flatten()
        .flat_map(|value| {
            value
                .split(':')
                .map(str::trim)
                .filter(|tag| !tag.is_empty())
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    tags.sort();
    tags.dedup();
    tags
}

fn extract_doc_type(metadata: &OrgDocumentMetadata) -> Option<DocumentType> {
    metadata
        .properties
        .get("TYPE")
        .or_else(|| metadata.properties.get("KIND"))
        .cloned()
        .or_else(|| first_keyword_value(metadata, "TYPE"))
        .or_else(|| first_keyword_value(metadata, "KIND"))
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .map(DocumentType::new)
}

fn first_keyword_value(metadata: &OrgDocumentMetadata, key: &str) -> Option<String> {
    metadata
        .keywords
        .get(key)
        .and_then(|values| values.first())
        .cloned()
}

fn extract_lead(body: &str) -> String {
    let mut in_property_block = false;
    for line in body.lines() {
        let trimmed = line.trim();
        if trimmed.eq_ignore_ascii_case(":PROPERTIES:") {
            in_property_block = true;
            continue;
        }
        if in_property_block {
            if trimmed.eq_ignore_ascii_case(":END:") {
                in_property_block = false;
            }
            continue;
        }
        if trimmed.is_empty()
            || trimmed.starts_with('*')
            || trimmed.starts_with("#+")
            || trimmed.starts_with(':')
        {
            continue;
        }
        return trimmed
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .chars()
            .take(LEAD_LIMIT)
            .collect();
    }
    String::new()
}
