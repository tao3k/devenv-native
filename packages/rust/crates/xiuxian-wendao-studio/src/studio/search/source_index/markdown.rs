use std::path::{Component, Path};

use crate::studio::analysis;
use crate::studio::types::{AnalysisNode, AnalysisNodeKind, AstSearchHit};
use xiuxian_wendao::parsers::markdown::extract_observations;
use xiuxian_wendao_parsers::parse_markdown_toc;
use xiuxian_wendao_parsers::sections::MarkdownSection;

use super::navigation::ast_navigation_target;

pub(super) fn markdown_scope_name(path: &Path) -> String {
    path.components()
        .find_map(|component| match component {
            Component::Normal(segment) => segment.to_str().map(ToString::to_string),
            _ => None,
        })
        .filter(|segment| !segment.is_empty())
        .unwrap_or_else(|| "docs".to_string())
}

pub(crate) fn build_markdown_ast_hits(
    _root: &Path,
    source_path: &Path,
    path: &str,
    content: &str,
    crate_name: &str,
) -> Vec<AstSearchHit> {
    let nodes = analysis::compile_markdown_nodes(path, content);
    let fallback_title = source_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .filter(|stem| !stem.is_empty())
        .unwrap_or("page");
    let parsed = parse_markdown_toc(content, fallback_title);
    build_markdown_ast_hits_from_sections(path, crate_name, &nodes, parsed.sections.as_slice())
}

pub(crate) fn build_markdown_ast_hits_from_sections(
    path: &str,
    crate_name: &str,
    nodes: &[AnalysisNode],
    sections: &[MarkdownSection],
) -> Vec<AstSearchHit> {
    let mut hits = build_markdown_node_hits(path, crate_name, nodes);
    for section in sections {
        hits.extend(build_markdown_property_hits_from_toc_section(
            path, crate_name, section,
        ));
        hits.extend(build_markdown_observation_hits_from_toc_section(
            path, crate_name, section,
        ));
    }
    hits
}

fn build_markdown_node_hits(
    path: &str,
    crate_name: &str,
    nodes: &[AnalysisNode],
) -> Vec<AstSearchHit> {
    nodes
        .iter()
        .filter_map(|node| {
            let signature = markdown_signature(node.kind, node.depth, node.label.as_str())?;
            Some(AstSearchHit {
                name: node.label.clone(),
                signature,
                path: path.to_string(),
                language: "markdown".to_string(),
                crate_name: crate_name.to_string(),
                project_name: None,
                root_label: None,
                node_kind: markdown_node_kind(node.kind).map(ToOwned::to_owned),
                owner_title: None,
                navigation_target: ast_navigation_target(
                    path,
                    crate_name,
                    None,
                    None,
                    node.line_start,
                    node.line_end,
                ),
                line_start: node.line_start,
                line_end: node.line_end,
                score: 0.0,
            })
        })
        .collect()
}

fn markdown_signature(kind: AnalysisNodeKind, depth: usize, label: &str) -> Option<String> {
    match kind {
        AnalysisNodeKind::Section => Some(format!("{} {label}", "#".repeat(depth.clamp(1, 6)))),
        AnalysisNodeKind::Task => Some(format!("- [ ] {label}")),
        _ => None,
    }
}

fn markdown_node_kind(kind: AnalysisNodeKind) -> Option<&'static str> {
    match kind {
        AnalysisNodeKind::Section => Some("section"),
        AnalysisNodeKind::Task => Some("task"),
        _ => None,
    }
}

fn build_markdown_property_hits_from_toc_section(
    path: &str,
    crate_name: &str,
    section: &MarkdownSection,
) -> Vec<AstSearchHit> {
    let owner_title = markdown_owner_title_from_toc_section(section);
    section
        .attributes()
        .iter()
        .filter(|(key, _)| !is_observation_attribute(key.as_str()))
        .map(|(key, value)| AstSearchHit {
            name: key.clone(),
            signature: format!(":{key}: {value}"),
            path: path.to_string(),
            language: "markdown".to_string(),
            crate_name: crate_name.to_string(),
            project_name: None,
            root_label: None,
            node_kind: Some("property".to_string()),
            owner_title: owner_title.clone(),
            navigation_target: ast_navigation_target(
                path,
                crate_name,
                None,
                None,
                section.line_start(),
                section.line_end(),
            ),
            line_start: section.line_start(),
            line_end: section.line_end(),
            score: 0.0,
        })
        .collect()
}

fn build_markdown_observation_hits_from_toc_section(
    path: &str,
    crate_name: &str,
    section: &MarkdownSection,
) -> Vec<AstSearchHit> {
    let owner_title = markdown_owner_title_from_toc_section(section);
    extract_observations(section.attributes())
        .into_iter()
        .map(|observation| AstSearchHit {
            name: "OBSERVE".to_string(),
            signature: format!(":OBSERVE: {}", observation.raw_value),
            path: path.to_string(),
            language: "markdown".to_string(),
            crate_name: crate_name.to_string(),
            project_name: None,
            root_label: None,
            node_kind: Some("observation".to_string()),
            owner_title: owner_title.clone(),
            navigation_target: ast_navigation_target(
                path,
                crate_name,
                None,
                None,
                section.line_start(),
                section.line_end(),
            ),
            line_start: section.line_start(),
            line_end: section.line_end(),
            score: 0.0,
        })
        .collect()
}

fn markdown_owner_title_from_toc_section(section: &MarkdownSection) -> Option<String> {
    if !section.heading_path().trim().is_empty() {
        Some(section.heading_path().to_string())
    } else if !section.heading_title().trim().is_empty() {
        Some(section.heading_title().to_string())
    } else {
        None
    }
}

fn is_observation_attribute(key: &str) -> bool {
    key == "OBSERVE" || key.starts_with("OBSERVE_")
}
