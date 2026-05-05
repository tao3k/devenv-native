//! Org section extraction from native Org syntax.

use std::collections::HashMap;

use orgize::rowan::ast::AstNode;
use orgize::{Org, ast::Headline};

use crate::sections::{SectionCore, SectionMetadata, SectionScope, extract_logbook_entries};

use super::types::OrgSection;

#[derive(Debug, Clone)]
struct OrgHeadline {
    title: String,
    level: usize,
    byte_start: usize,
    content_byte_start: usize,
    line_start: usize,
    attributes: HashMap<String, String>,
}

/// Extract parser-owned Org sections from one document body.
#[must_use]
pub fn extract_org_sections(body: &str) -> Vec<OrgSection> {
    let org = Org::parse(body);
    let headlines = org
        .document()
        .syntax()
        .descendants()
        .filter_map(Headline::cast)
        .map(|headline| org_headline(body, &headline))
        .collect::<Vec<_>>();

    if headlines.is_empty() {
        return vec![root_section(body)];
    }

    let mut sections = Vec::new();
    push_root_section(&mut sections, body, headlines.first());

    let mut heading_stack = Vec::<String>::new();
    for (index, headline) in headlines.iter().enumerate() {
        if heading_stack.len() >= headline.level {
            heading_stack.truncate(headline.level.saturating_sub(1));
        }
        heading_stack.push(headline.title.clone());
        let heading_path = heading_stack.join(" / ");
        let byte_end = headlines
            .get(index + 1)
            .map_or(body.len(), |next| next.byte_start)
            .max(headline.content_byte_start);
        let section_text = body[headline.content_byte_start..byte_end]
            .trim()
            .to_string();
        let line_end = if byte_end == 0 {
            headline.line_start
        } else {
            line_number_for_byte(body, byte_end.saturating_sub(1)).max(headline.line_start)
        };
        let lines = section_text
            .lines()
            .map(ToString::to_string)
            .collect::<Vec<_>>();
        let logbook = extract_logbook_entries(&lines, headline.line_start.saturating_add(1));

        sections.push(SectionCore {
            scope: SectionScope {
                heading_title: headline.title.clone(),
                heading_path: heading_path.clone(),
                heading_path_lower: heading_path.to_lowercase(),
                heading_level: headline.level,
                line_start: headline.line_start,
                line_end,
                byte_start: headline.byte_start,
                byte_end,
            },
            section_text_lower: section_text.to_lowercase(),
            section_text,
            metadata: SectionMetadata {
                attributes: headline.attributes.clone(),
                logbook,
            },
        });
    }

    sections
}

fn org_headline(body: &str, headline: &Headline) -> OrgHeadline {
    let byte_start = usize::from(headline.start()).min(body.len());
    let content_byte_start = next_line_start(body, byte_start);
    let attributes = headline
        .properties()
        .map(|drawer| {
            drawer
                .iter()
                .filter_map(|(key, value)| normalize_property_pair(key.as_ref(), value.as_ref()))
                .collect()
        })
        .unwrap_or_default();
    OrgHeadline {
        title: headline.title_raw().trim().to_string(),
        level: headline.level(),
        byte_start,
        content_byte_start,
        line_start: line_number_for_byte(body, byte_start),
        attributes,
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

fn next_line_start(body: &str, byte_start: usize) -> usize {
    body[byte_start..]
        .find('\n')
        .map_or(body.len(), |relative| byte_start + relative + 1)
}

fn line_number_for_byte(body: &str, byte_offset: usize) -> usize {
    body[..byte_offset.min(body.len())]
        .bytes()
        .filter(|byte| *byte == b'\n')
        .count()
        + 1
}

fn push_root_section(sections: &mut Vec<OrgSection>, body: &str, first: Option<&OrgHeadline>) {
    let byte_end = first.map_or(body.len(), |headline| headline.byte_start);
    let section_text = body[..byte_end].trim().to_string();
    if section_text.is_empty() {
        return;
    }
    sections.push(SectionCore {
        scope: SectionScope {
            heading_title: String::new(),
            heading_path: String::new(),
            heading_path_lower: String::new(),
            heading_level: 0,
            line_start: 1,
            line_end: line_number_for_byte(body, byte_end.saturating_sub(1)).max(1),
            byte_start: 0,
            byte_end,
        },
        section_text_lower: section_text.to_lowercase(),
        section_text,
        metadata: SectionMetadata {
            attributes: HashMap::new(),
            logbook: Vec::new(),
        },
    });
}

fn root_section(body: &str) -> OrgSection {
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
