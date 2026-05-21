use crate::link_graph::index::IndexedSection;
use crate::link_graph::models::{PageIndexMeta, PageIndexNode};
use std::collections::HashMap;
use std::sync::Arc;
use xiuxian_wendao_parsers::blocks::extract_blocks;

/// Build a deterministic page tree for one document from flat parsed sections.
pub(crate) fn build_page_index_tree(
    doc_id: &str,
    doc_title: &str,
    sections: &[IndexedSection],
) -> Vec<PageIndexNode> {
    let mut roots = Vec::new();
    let mut stack = Vec::new();
    let mut slug_counts = HashMap::new();
    let has_named_headings = sections
        .iter()
        .any(|section| !section.heading_path.trim().is_empty());

    sections
        .iter()
        .filter(|section| has_page_index_content(section))
        .for_each(|section| {
            let level = effective_level(section.heading_level);
            while stack
                .last()
                .is_some_and(|parent: &PageIndexNode| parent.level >= level)
            {
                close_last_open_node(&mut roots, &mut stack);
            }

            let parent_id = stack.last().map(|p| p.node_id.clone());
            let node = build_node(
                doc_id,
                doc_title,
                section,
                has_named_headings,
                &mut slug_counts,
                parent_id,
            );
            stack.push(node);
        });

    while !stack.is_empty() {
        close_last_open_node(&mut roots, &mut stack);
    }

    roots
}

fn has_page_index_content(section: &IndexedSection) -> bool {
    !section.section_text.trim().is_empty() || !section.heading_path.trim().is_empty()
}

fn build_node(
    doc_id: &str,
    doc_title: &str,
    section: &IndexedSection,
    has_named_headings: bool,
    slug_counts: &mut HashMap<String, usize>,
    parent_id: Option<String>,
) -> PageIndexNode {
    let title = effective_title(section, doc_title, has_named_headings);
    let slug = effective_slug(section, &title);

    // Check for explicit :ID: attribute to use as anchor
    let explicit_id = section.attributes.get("ID");
    let node_id = if let Some(id) = explicit_id {
        // Use explicit ID: doc_id#explicit-id
        format!("{doc_id}#{id}")
    } else {
        // Generate deterministic ID from slug
        let sequence = slug_counts.entry(slug.clone()).or_insert(0);
        *sequence += 1;
        if *sequence == 1 {
            format!("{doc_id}#{slug}")
        } else {
            format!("{doc_id}#{slug}-{}", *sequence - 1)
        }
    };

    let line_start = section.line_start;
    let line_end = section.line_end;

    // Build structural path from heading hierarchy
    let structural_path = if section.heading_path.is_empty() {
        Vec::new()
    } else {
        section
            .heading_path
            .split(" / ")
            .map(|s| s.trim().to_string())
            .collect()
    };

    // Calculate content hash (Blake3 truncated to 16 chars)
    let content_hash = calculate_content_hash(&section.section_text);

    // Extract block-level elements for fine-grained addressing
    let blocks = extract_blocks(
        &section.section_text,
        section.byte_start,
        section.line_start,
        &structural_path,
    );

    PageIndexNode {
        node_id,
        parent_id,
        title,
        level: effective_level(section.heading_level),
        text: Arc::<str>::from(section.section_text.as_str()),
        summary: None,
        children: Vec::new(),
        blocks,
        metadata: PageIndexMeta {
            line_range: (line_start, line_end),
            byte_range: Some((section.byte_start, section.byte_end)),
            structural_path,
            content_hash: Some(content_hash),
            attributes: section.attributes.clone(),
            token_count: count_tokens(&section.section_text),
            is_thinned: false,
            logbook: section.logbook.clone(),
            observations: section.observations.clone(),
        },
    }
}

/// Calculate Blake3 content hash, truncated to 16 hex characters.
fn calculate_content_hash(text: &str) -> String {
    use blake3::Hasher;
    let mut hasher = Hasher::new();
    hasher.update(text.as_bytes());
    let hash = hasher.finalize();
    // Use first 8 bytes (16 hex chars) for compact storage
    hash.to_hex()[..16].to_string()
}

fn close_last_open_node(roots: &mut Vec<PageIndexNode>, stack: &mut Vec<PageIndexNode>) {
    let Some(node) = stack.pop() else {
        return;
    };
    if let Some(parent) = stack.last_mut() {
        parent.children.push(node);
    } else {
        roots.push(node);
    }
}

fn effective_title(section: &IndexedSection, doc_title: &str, has_named_headings: bool) -> String {
    if !section.heading_title.trim().is_empty() {
        return section.heading_title.clone();
    }
    if !section.heading_path.trim().is_empty() {
        return section
            .heading_path
            .rsplit(" / ")
            .next()
            .unwrap_or(doc_title)
            .to_string();
    }
    if has_named_headings {
        return "Overview".to_string();
    }
    doc_title.to_string()
}

fn effective_level(level: usize) -> usize {
    level.clamp(1, 6)
}

fn effective_slug(section: &IndexedSection, title: &str) -> String {
    let raw = if section.heading_path_lower.trim().is_empty() {
        title
    } else {
        section.heading_path_lower.as_str()
    };
    let slug = raw
        .chars()
        .map(|ch| match ch {
            'a'..='z' | '0'..='9' => ch,
            'A'..='Z' => ch.to_ascii_lowercase(),
            _ => '-',
        })
        .collect::<String>();
    let slug = slug.trim_matches('-').to_string();
    if slug.is_empty() {
        "overview".to_string()
    } else {
        slug
    }
}

fn count_tokens(text: &str) -> usize {
    text.split_whitespace().count()
}
