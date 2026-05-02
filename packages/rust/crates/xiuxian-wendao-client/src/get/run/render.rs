//! Text and JSON rendering for local `wendao get` projections.

use anyhow::{Context, Result};

use super::{
    DocsPageIndexDocumentsResult, DocsPageIndexTreesResult, OutputFormat, ProjectedPageIndexLink,
    ProjectedPageIndexNode, effective_section_level,
};

pub(super) fn emit_toc_output(
    result: &DocsPageIndexDocumentsResult,
    output: OutputFormat,
) -> Result<()> {
    let rendered = match output {
        OutputFormat::Text => render_toc_markdown(result),
        OutputFormat::Json => {
            serde_json::to_string(result).context("failed to serialize get output as JSON")?
        }
        OutputFormat::Pretty => serde_json::to_string_pretty(result)
            .context("failed to serialize get output as JSON")?,
    };
    println!("{rendered}");
    Ok(())
}

pub(super) fn emit_page_index_output(
    result: &DocsPageIndexTreesResult,
    output: OutputFormat,
) -> Result<()> {
    let rendered = match output {
        OutputFormat::Text => render_page_index_markdown(result),
        OutputFormat::Json => {
            serde_json::to_string(result).context("failed to serialize get output as JSON")?
        }
        OutputFormat::Pretty => serde_json::to_string_pretty(result)
            .context("failed to serialize get output as JSON")?,
    };
    println!("{rendered}");
    Ok(())
}

pub(super) fn render_toc_markdown(result: &DocsPageIndexDocumentsResult) -> String {
    if result.documents.is_empty() {
        return "_No documents matched._".to_string();
    }

    let mut lines = Vec::new();
    for (index, document) in result.documents.iter().enumerate() {
        if index > 0 {
            lines.push(String::new());
        }
        lines.push(format!("path: {}", document.path));
        lines.push(format!(
            "title: {} | sections: {}",
            document.title,
            document.sections.len()
        ));
        for section in &document.sections {
            let title = if section.title.trim().is_empty() {
                "(untitled)"
            } else {
                section.title.as_str()
            };
            lines.push(render_heading_with_range(
                section.level,
                title,
                section.line_range,
            ));
        }
    }
    lines.join("\n")
}

fn render_heading_with_range(level: usize, title: &str, line_range: (usize, usize)) -> String {
    let marker_count = effective_section_level(level);
    format!(
        "{} {} -> [L{} {}-{}]",
        "#".repeat(marker_count),
        title,
        effective_section_level(level),
        line_range.0,
        line_range.1
    )
}

pub(super) fn render_page_index_markdown(result: &DocsPageIndexTreesResult) -> String {
    if result.trees.is_empty() {
        return "_No documents matched._".to_string();
    }

    let mut lines = Vec::new();
    for (index, tree) in result.trees.iter().enumerate() {
        if index > 0 {
            lines.push(String::new());
        }
        lines.push(format!("path: {}", tree.path));
        lines.push(format!(
            "kind: {:?} | roots: {} | nodes: {} | links: {} | embeds: {}",
            tree.kind,
            tree.root_count,
            count_tree_nodes(tree.roots.as_slice()),
            count_tree_links(tree.roots.as_slice()),
            count_tree_embeds(tree.roots.as_slice())
        ));
        for root in &tree.roots {
            push_tree_markdown_lines(&mut lines, root);
        }
    }
    lines.join("\n")
}

fn push_tree_markdown_lines(lines: &mut Vec<String>, node: &ProjectedPageIndexNode) {
    lines.push(render_heading_with_range(
        node.level,
        node.title.as_str(),
        node.line_range,
    ));
    let section_links = node
        .links
        .iter()
        .filter(|link| !projected_page_index_link_is_embed(link))
        .cloned()
        .collect::<Vec<_>>();
    if !section_links.is_empty() {
        lines.push(format!(
            "links: {}",
            render_node_link_surfaces(section_links.as_slice())
        ));
    }
    let section_embeds = node
        .links
        .iter()
        .filter(|link| projected_page_index_link_is_embed(link))
        .cloned()
        .collect::<Vec<_>>();
    if !section_embeds.is_empty() {
        lines.push(format!(
            "embeds: {}",
            render_node_link_surfaces(section_embeds.as_slice())
        ));
    }
    for child in &node.children {
        push_tree_markdown_lines(lines, child);
    }
}

fn count_tree_nodes(nodes: &[ProjectedPageIndexNode]) -> usize {
    nodes
        .iter()
        .map(|node| 1 + count_tree_nodes(node.children.as_slice()))
        .sum()
}

fn count_tree_links(nodes: &[ProjectedPageIndexNode]) -> usize {
    nodes
        .iter()
        .map(|node| {
            node.links
                .iter()
                .filter(|link| !projected_page_index_link_is_embed(link))
                .count()
                + count_tree_links(node.children.as_slice())
        })
        .sum()
}

fn count_tree_embeds(nodes: &[ProjectedPageIndexNode]) -> usize {
    nodes
        .iter()
        .map(|node| {
            node.links
                .iter()
                .filter(|link| projected_page_index_link_is_embed(link))
                .count()
                + count_tree_embeds(node.children.as_slice())
        })
        .sum()
}

fn projected_page_index_link_is_embed(link: &ProjectedPageIndexLink) -> bool {
    matches!(link.kind.as_str(), "markdown_image" | "wiki_embed")
}

fn render_node_link_surfaces(links: &[ProjectedPageIndexLink]) -> String {
    links
        .iter()
        .map(|link| {
            if link.surface.trim().is_empty() {
                link.target.clone()
            } else {
                link.surface.clone()
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}
