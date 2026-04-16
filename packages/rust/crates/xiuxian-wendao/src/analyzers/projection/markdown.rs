use std::path::Path;

use crate::analyzers::RepoIntelligenceError;
use crate::analyzers::RepositoryAnalysisOutput;
use crate::link_graph::{
    DEFAULT_PAGE_INDEX_THINNING_TOKEN_THRESHOLD, IndexedSection, PageIndexNode,
    build_page_index_tree, thin_page_index_tree,
};
use xiuxian_wendao_parsers::parse_markdown_toc;

use super::contracts::{
    ProjectedMarkdownDocument, ProjectedPageIndexDocument, ProjectedPageIndexNode,
    ProjectedPageIndexSection, ProjectedPageIndexTree, ProjectionPageKind,
};
use super::pages::build_projected_pages;

struct ParsedProjectedDocument {
    document: ProjectedMarkdownDocument,
    doc_id: String,
    title: String,
    indexed_sections: Vec<IndexedSection>,
    section_summaries: Vec<ProjectedPageIndexSection>,
}

/// Render projected page records into deterministic virtual markdown documents.
#[must_use]
pub fn render_projected_markdown_documents(
    analysis: &RepositoryAnalysisOutput,
) -> Vec<ProjectedMarkdownDocument> {
    build_projected_pages(analysis)
        .into_iter()
        .map(|page| ProjectedMarkdownDocument {
            repo_id: page.repo_id.clone(),
            page_id: page.page_id.clone(),
            kind: page.kind,
            path: projected_markdown_path(page.kind, page.title.as_str(), page.page_id.as_str()),
            title: page.title.clone(),
            markdown: render_projected_markdown(
                page.title.as_str(),
                page.kind,
                page.format_hints.as_slice(),
                page.sections.as_slice(),
            ),
        })
        .collect()
}

/// Build page-index-ready parsed documents from deterministic projected markdown.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError::AnalysisFailed`] when a projected markdown
/// document cannot be parsed through the existing markdown parser.
pub fn build_projected_page_index_documents(
    analysis: &RepositoryAnalysisOutput,
) -> Result<Vec<ProjectedPageIndexDocument>, RepoIntelligenceError> {
    parse_projected_documents(analysis)?
        .into_iter()
        .map(|parsed| {
            Ok(ProjectedPageIndexDocument {
                repo_id: parsed.document.repo_id,
                page_id: parsed.document.page_id,
                path: parsed.document.path,
                doc_id: parsed.doc_id,
                title: parsed.title,
                sections: parsed.section_summaries,
            })
        })
        .collect()
}

/// Build real page-index trees for projected pages using Wendao's page-index builder.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError::AnalysisFailed`] when a projected markdown
/// document cannot be parsed through the existing markdown parser.
pub fn build_projected_page_index_trees(
    analysis: &RepositoryAnalysisOutput,
) -> Result<Vec<ProjectedPageIndexTree>, RepoIntelligenceError> {
    parse_projected_documents(analysis)?
        .into_iter()
        .map(|parsed| {
            let mut tree = build_page_index_tree(
                parsed.doc_id.as_str(),
                parsed.title.as_str(),
                &parsed.indexed_sections,
            );
            thin_page_index_tree(&mut tree, DEFAULT_PAGE_INDEX_THINNING_TOKEN_THRESHOLD);
            Ok(ProjectedPageIndexTree {
                repo_id: parsed.document.repo_id,
                page_id: parsed.document.page_id,
                kind: parsed.document.kind,
                path: parsed.document.path,
                doc_id: parsed.doc_id,
                title: parsed.title,
                root_count: tree.len(),
                roots: tree.iter().map(snapshot_page_index_node).collect(),
            })
        })
        .collect()
}

fn projected_markdown_path(kind: ProjectionPageKind, title: &str, page_id: &str) -> String {
    let slug = slugify(title);
    let hash = blake3::hash(page_id.as_bytes()).to_hex().to_string();
    format!("{}/{slug}-{}.md", projection_kind_token(kind), &hash[..12])
}

fn parse_projected_documents(
    analysis: &RepositoryAnalysisOutput,
) -> Result<Vec<ParsedProjectedDocument>, RepoIntelligenceError> {
    render_projected_markdown_documents(analysis)
        .into_iter()
        .map(|document| {
            let toc = parse_markdown_toc(
                document.markdown.as_str(),
                projected_fallback_title(document.path.as_str()),
            );
            let doc_id = projected_doc_id(document.path.as_str()).ok_or_else(|| {
                RepoIntelligenceError::AnalysisFailed {
                    message: format!(
                        "failed to derive projected markdown doc id `{}` for `{}`",
                        document.path, document.page_id
                    ),
                }
            })?;
            let section_summaries = toc
                .sections
                .iter()
                .map(|section| ProjectedPageIndexSection {
                    heading_path: section.scope.heading_path.clone(),
                    title: section.scope.heading_title.clone(),
                    level: section.scope.heading_level,
                    line_range: (section.scope.line_start, section.scope.line_end),
                    attributes: sorted_attributes(section.metadata.attributes.clone()),
                })
                .collect();
            let indexed_sections = toc
                .sections
                .iter()
                .map(IndexedSection::from_markdown_section)
                .collect::<Vec<_>>();
            Ok(ParsedProjectedDocument {
                document,
                doc_id,
                title: toc.document.core.title,
                indexed_sections,
                section_summaries,
            })
        })
        .collect()
}

fn render_projected_markdown(
    title: &str,
    kind: ProjectionPageKind,
    format_hints: &[String],
    sections: &[super::contracts::ProjectedPageSection],
) -> String {
    let mut markdown = String::new();
    markdown.push_str("# ");
    markdown.push_str(title);
    markdown.push_str("\n:PROPERTIES:\n");
    markdown.push_str(":TYPE: PROJECTED_PAGE\n");
    markdown.push_str(":KIND: ");
    markdown.push_str(projection_kind_token(kind));
    markdown.push('\n');
    if !format_hints.is_empty() {
        markdown.push_str(":FORMATS: ");
        markdown.push_str(format_hints.join(", ").as_str());
        markdown.push('\n');
    }
    markdown.push_str(":END:\n\n");

    for (index, section) in sections.iter().enumerate() {
        markdown.push_str(&"#".repeat(section.level.clamp(1, 6)));
        markdown.push(' ');
        markdown.push_str(section.title.as_str());
        markdown.push('\n');
        markdown.push_str(":ID: ");
        markdown.push_str(section_anchor_id(section.section_id.as_str()).as_str());
        markdown.push('\n');
        if !section.paths.is_empty() {
            markdown.push_str(":PATHS: ");
            markdown.push_str(section.paths.join(", ").as_str());
            markdown.push('\n');
        }
        markdown.push('\n');
        markdown.push_str(section.body.as_str());
        markdown.push('\n');
        if index + 1 != sections.len() {
            markdown.push('\n');
        }
    }

    markdown
}

fn section_anchor_id(section_id: &str) -> String {
    section_id
        .rsplit('#')
        .next()
        .unwrap_or(section_id)
        .to_string()
}

fn projection_kind_token(kind: ProjectionPageKind) -> &'static str {
    match kind {
        ProjectionPageKind::Reference => "reference",
        ProjectionPageKind::HowTo => "howto",
        ProjectionPageKind::Tutorial => "tutorial",
        ProjectionPageKind::Explanation => "explanation",
    }
}

fn projected_doc_id(path: &str) -> Option<String> {
    let normalized = path.replace('\\', "/");
    let lower = normalized.to_ascii_lowercase();
    let without_ext = [".markdown", ".mdx", ".md"]
        .into_iter()
        .find_map(|ext| {
            lower
                .ends_with(ext)
                .then(|| normalized[..normalized.len() - ext.len()].to_string())
        })
        .unwrap_or(normalized);
    let out = without_ext.trim_matches('/').to_string();
    if out.is_empty() { None } else { Some(out) }
}

fn projected_fallback_title(path: &str) -> &str {
    Path::new(path)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .filter(|stem| !stem.is_empty())
        .unwrap_or("page")
}

fn slugify(raw: &str) -> String {
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
        "page".to_string()
    } else {
        slug
    }
}

fn sorted_attributes(
    attributes: std::collections::HashMap<String, String>,
) -> Vec<(String, String)> {
    let mut attributes = attributes.into_iter().collect::<Vec<_>>();
    attributes.sort_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.cmp(&right.1)));
    attributes
}

fn snapshot_page_index_node(node: &PageIndexNode) -> ProjectedPageIndexNode {
    ProjectedPageIndexNode {
        node_id: node.node_id.clone(),
        title: node.title.clone(),
        level: node.level,
        structural_path: node.metadata.structural_path.clone(),
        line_range: node.metadata.line_range,
        token_count: node.metadata.token_count,
        is_thinned: node.metadata.is_thinned,
        text: node.text.to_string(),
        summary: heuristic_summary(&node.text),
        children: node.children.iter().map(snapshot_page_index_node).collect(),
    }
}

fn heuristic_summary(text: &str) -> Option<String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }
    // Take the first sentence or first 120 chars
    let first_sentence = trimmed.split(['.', '!', '?']).next().unwrap_or(trimmed);

    let summary = if first_sentence.len() > 120 {
        format!("{}...", &first_sentence[..117])
    } else {
        first_sentence.to_string()
    };

    Some(summary)
}
