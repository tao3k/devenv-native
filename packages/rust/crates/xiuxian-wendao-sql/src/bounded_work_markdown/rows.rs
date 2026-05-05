use std::collections::HashMap;
use std::fs;
use std::path::Path;

use xiuxian_wendao_parsers::{DocumentCore, parse_markdown_toc, parse_org_toc};

use super::discovery::DiscoveredMarkdownFile;
use super::skeleton::render_markdown_skeleton;

/// One queryable structural markdown unit inside a bounded work surface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundedWorkMarkdownRow {
    /// The markdown file path relative to the bounded-work root.
    pub path: String,
    /// The top-level bounded-work surface that owns this row.
    pub surface: String,
    /// The first-order kind derived from the bounded surface and relative path.
    pub surface_kind: String,
    /// The normalized heading path using `/` as the segment separator.
    pub heading_path: String,
    /// The current row title or effective section title.
    pub title: String,
    /// The heading level for this row, or `0` for the document root row.
    pub level: i64,
    /// The structural compression view used for low-token reads.
    pub skeleton: String,
    /// The full markdown body for this structural unit.
    pub body: String,
}

pub(crate) fn build_rows_for_file(
    _root: &Path,
    file: &DiscoveredMarkdownFile,
) -> Result<Vec<BoundedWorkMarkdownRow>, String> {
    let body = fs::read_to_string(&file.absolute_path).map_err(|error| {
        format!(
            "failed to read bounded work markdown file `{}`: {error}",
            file.absolute_path.display()
        )
    })?;
    let fallback_title = file
        .absolute_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .filter(|stem| !stem.is_empty())
        .unwrap_or("page");
    let (document, sections) =
        parse_bounded_work_document(&file.absolute_path, &body, fallback_title);
    let surface_kind = classify_surface_kind(file);

    let mut rows = Vec::with_capacity(sections.len() + 1);
    rows.push(BoundedWorkMarkdownRow {
        path: file.relative_path.clone(),
        surface: file.surface.clone(),
        surface_kind: surface_kind.clone(),
        heading_path: String::new(),
        title: document.title.clone(),
        level: 0,
        skeleton: render_markdown_skeleton(&document.title, 1, &HashMap::new(), &body),
        body: body.trim().to_string(),
    });

    rows.extend(sections.iter().map(|section| {
        let heading_path = normalize_heading_path(section.scope.heading_path.as_str());
        let title = effective_title(heading_path.as_str(), section.scope.heading_title.as_str());
        BoundedWorkMarkdownRow {
            path: file.relative_path.clone(),
            surface: file.surface.clone(),
            surface_kind: surface_kind.clone(),
            heading_path,
            title: title.clone(),
            level: i64::try_from(section.scope.heading_level).unwrap_or(i64::MAX),
            skeleton: render_markdown_skeleton(
                title.as_str(),
                section.scope.heading_level.max(1),
                &section.metadata.attributes,
                section.section_text.as_str(),
            ),
            body: section.section_text.trim().to_string(),
        }
    }));

    Ok(rows)
}

fn parse_bounded_work_document(
    path: &Path,
    body: &str,
    fallback_title: &str,
) -> (
    DocumentCore,
    Vec<xiuxian_wendao_parsers::sections::MarkdownSection>,
) {
    if path
        .extension()
        .and_then(|value| value.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("org"))
    {
        let parsed = parse_org_toc(body, fallback_title);
        return (parsed.document.core, parsed.sections);
    }
    let parsed = parse_markdown_toc(body, fallback_title);
    (parsed.document.core, parsed.sections)
}

fn classify_surface_kind(file: &DiscoveredMarkdownFile) -> String {
    match file.surface.as_str() {
        "blueprint" => "blueprint_note".to_string(),
        "plan" => "plan_note".to_string(),
        "semantic" => classify_semantic_path(file.relative_path.as_str()).to_string(),
        surface => format!("{surface}_note"),
    }
}

fn classify_semantic_path(relative_path: &str) -> &'static str {
    if relative_path.starts_with("semantic/objects/") {
        return "semantic_object";
    }
    if relative_path.starts_with("semantic/projections/") {
        return "semantic_projection";
    }
    if relative_path.starts_with("semantic/change-intents/") {
        return "semantic_change_intent";
    }
    "semantic_note"
}

fn normalize_heading_path(raw: &str) -> String {
    raw.split(" / ")
        .map(str::trim)
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>()
        .join("/")
}

fn effective_title(heading_path: &str, heading_title: &str) -> String {
    if !heading_title.trim().is_empty() {
        return heading_title.trim().to_string();
    }
    heading_path
        .rsplit('/')
        .next()
        .map(str::trim)
        .filter(|segment| !segment.is_empty())
        .unwrap_or("Overview")
        .to_string()
}
