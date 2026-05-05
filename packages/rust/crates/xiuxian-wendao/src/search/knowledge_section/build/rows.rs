use std::path::Path;

use crate::parsers::markdown::{ParsedNote, ParsedSection};
use crate::search::MarkdownSnapshotEntry;
use crate::search::contracts::SearchProjectMetadata;
use crate::search::contracts::{SearchHit, SearchProjectConfig, StudioNavigationTarget};
use crate::search::knowledge_section::build::paths::studio_display_path;
use crate::search::knowledge_section::schema::KnowledgeSectionRow;

pub(super) fn build_knowledge_section_rows_for_entry(
    project_root: &Path,
    config_root: &Path,
    projects: &[SearchProjectConfig],
    entry: &MarkdownSnapshotEntry,
) -> Vec<KnowledgeSectionRow> {
    let Some(parsed) = entry.parsed_note.as_ref() else {
        return Vec::new();
    };
    let metadata = SearchProjectMetadata {
        project_name: entry.file.project_name.clone(),
        root_label: entry.file.root_label.clone(),
    };
    knowledge_rows_for_note(project_root, config_root, projects, parsed, &metadata)
}

fn knowledge_rows_for_note(
    project_root: &Path,
    config_root: &Path,
    projects: &[SearchProjectConfig],
    parsed: &ParsedNote,
    metadata: &SearchProjectMetadata,
) -> Vec<KnowledgeSectionRow> {
    let display_path = studio_display_path(
        project_root,
        config_root,
        projects,
        metadata,
        parsed.doc.path.as_str(),
    );
    let hierarchy = Some(
        display_path
            .split('/')
            .filter(|segment| !segment.is_empty())
            .map(str::to_string)
            .collect::<Vec<_>>(),
    );
    let navigation_target = Some(StudioNavigationTarget {
        path: parsed.doc.path.clone(),
        category: "doc".to_string(),
        project_name: metadata.project_name.clone(),
        root_label: metadata.root_label.clone(),
        line: None,
        line_end: None,
        column: None,
    });
    let mut tags = parsed.doc.tags.clone();
    if let Some(doc_type) = parsed.doc.doc_type.as_deref()
        && !tags.iter().any(|tag| tag == doc_type)
    {
        tags.push(doc_type.to_string());
    }

    let doc_hit = SearchHit {
        stem: parsed.doc.stem.clone(),
        title: Some(parsed.doc.title.clone()),
        path: display_path.clone(),
        doc_type: parsed.doc.doc_type.clone(),
        tags: tags.clone(),
        score: 0.0,
        best_section: None,
        match_reason: Some("knowledge_section_search".to_string()),
        hierarchical_uri: None,
        hierarchy: hierarchy.clone(),
        saliency_score: None,
        audit_status: None,
        verification_state: None,
        implicit_backlinks: None,
        implicit_backlink_items: None,
        navigation_target: navigation_target.clone(),
    };
    let doc_search_text = normalize_search_text(
        std::iter::once(parsed.doc.title.as_str())
            .chain(std::iter::once(parsed.doc.stem.as_str()))
            .chain(parsed.doc.doc_type.iter().map(String::as_str))
            .chain(tags.iter().map(String::as_str))
            .chain(parsed.sections.iter().flat_map(|section| {
                [
                    section.heading_title.as_str(),
                    section.heading_path.as_str(),
                    section.section_text.as_str(),
                ]
            })),
    );
    let mut rows = vec![doc_row_for_note(&display_path, &doc_hit, doc_search_text)];
    rows.extend(parsed.sections.iter().map(|section| {
        section_row_for_note(
            &display_path,
            parsed,
            section,
            &tags,
            hierarchy.clone(),
            navigation_target.clone(),
        )
    }));

    rows
}

fn doc_row_for_note(
    display_path: &str,
    doc_hit: &SearchHit,
    doc_search_text: String,
) -> KnowledgeSectionRow {
    KnowledgeSectionRow {
        id: format!("{display_path}:_doc"),
        path: display_path.to_string(),
        stem: doc_hit.stem.clone(),
        title: doc_hit.title.clone(),
        best_section: None,
        search_text: doc_search_text,
        hit_json: serialize_hit(doc_hit),
    }
}

fn section_row_for_note(
    display_path: &str,
    parsed: &ParsedNote,
    section: &ParsedSection,
    tags: &[String],
    hierarchy: Option<Vec<String>>,
    navigation_target: Option<StudioNavigationTarget>,
) -> KnowledgeSectionRow {
    let hit = SearchHit {
        stem: parsed.doc.stem.clone(),
        title: Some(parsed.doc.title.clone()),
        path: display_path.to_string(),
        doc_type: parsed.doc.doc_type.clone(),
        tags: tags.to_vec(),
        score: 0.0,
        best_section: Some(section.heading_path.clone()),
        match_reason: Some("knowledge_section_search".to_string()),
        hierarchical_uri: None,
        hierarchy,
        saliency_score: None,
        audit_status: None,
        verification_state: None,
        implicit_backlinks: None,
        implicit_backlink_items: None,
        navigation_target,
    };
    KnowledgeSectionRow {
        id: format!("{display_path}:{}", section.heading_path),
        path: display_path.to_string(),
        stem: parsed.doc.stem.clone(),
        title: Some(parsed.doc.title.clone()),
        best_section: Some(section.heading_path.clone()),
        search_text: normalize_search_text([
            parsed.doc.title.as_str(),
            parsed.doc.stem.as_str(),
            section.heading_title.as_str(),
            section.heading_path.as_str(),
            section.section_text.as_str(),
        ]),
        hit_json: serialize_hit(&hit),
    }
}

fn serialize_hit(hit: &SearchHit) -> String {
    serde_json::to_string(hit).unwrap_or_else(|error| {
        panic!("serialize knowledge hit should succeed: {error}");
    })
}

fn normalize_search_text<'a>(segments: impl IntoIterator<Item = &'a str>) -> String {
    let mut normalized = String::new();
    for segment in segments {
        let trimmed = segment.trim();
        if trimmed.is_empty() {
            continue;
        }
        if !normalized.is_empty() {
            normalized.push(' ');
        }
        normalized.push_str(trimmed);
        if normalized.len() >= 16 * 1024 {
            normalized.truncate(16 * 1024);
            break;
        }
    }
    normalized
}
