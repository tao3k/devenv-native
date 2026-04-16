//! Agentic Navigation Tool for Reasoning-Driven Discovery (Blueprint v2.4 Section 1).
//!
//! This module implements `wendao.agentic_nav`, a tool that provides "structured GPS"
//! for agents navigating the knowledge graph. It combines vector similarity search
//! with AST-guided validation to compute recommended exploration paths.

use schemars::JsonSchema;
use serde::Deserialize;
use xiuxian_zhenfa::{ZhenfaContext, ZhenfaError, zhenfa_tool};

use crate::link_graph::{
    LinkGraphIndex, LinkGraphSearchOptions, QuantumAnchorHit, RegistryIndex, SkeletonRerankOptions,
    TopologyIndex, skeleton_rerank,
};

use super::WendaoContextExt;

/// Arguments for the `wendao.agentic_nav` tool.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub(crate) struct WendaoAgenticNavArgs {
    /// The task intent or query to navigate.
    pub query: String,
    /// Optional document context to scope the navigation (filters results to this doc).
    #[serde(default)]
    pub doc_id: Option<String>,
    /// Maximum number of navigation candidates to return.
    #[serde(default = "default_limit")]
    pub limit: usize,
    /// Whether to use strict mode (filter out invalid anchors).
    #[serde(default)]
    pub strict: bool,
}

fn default_limit() -> usize {
    10
}

/// Agentic navigation tool for reasoning-driven discovery.
///
/// This tool provides "structured GPS" for agents navigating the knowledge graph.
/// It combines vector similarity search with AST-guided validation to compute
/// recommended exploration paths based on task intent.
#[allow(missing_docs)]
#[allow(clippy::needless_pass_by_value)]
#[zhenfa_tool(
    name = "wendao.agentic_nav",
    description = "Navigate the knowledge graph with reasoning-driven discovery. Combines vector search with AST validation for structured exploration paths.",
    tool_struct = "WendaoAgenticNavTool"
)]
pub fn wendao_agentic_nav(
    ctx: &ZhenfaContext,
    args: WendaoAgenticNavArgs,
) -> Result<String, ZhenfaError> {
    let query = args.query.trim();
    if query.is_empty() {
        return Err(ZhenfaError::invalid_arguments(
            "`query` must be a non-empty string",
        ));
    }

    let index = ctx.link_graph_index()?;
    let limit = args.limit.clamp(1, 100);

    // Build search options with optional doc_id filter
    let search_options = if let Some(ref doc_id) = args.doc_id {
        let mut opts = LinkGraphSearchOptions::default();
        opts.filters.include_paths.push(doc_id.clone());
        opts
    } else {
        LinkGraphSearchOptions::default()
    };

    // Perform search to get initial candidates
    let payload = index.search_planned_payload_with_agentic(
        query,
        limit * 3, // Get more for filtering
        search_options,
        None, // include_provisional
        None, // provisional_limit
    );

    // Build dual indices for validation from payload results
    let trees = build_page_index_trees_from_hits(&payload.results, &index);
    let registry = RegistryIndex::build_from_trees(&trees);
    let topology = TopologyIndex::build_from_trees(&trees);

    // Configure re-ranking options
    let rerank_options = if args.strict {
        SkeletonRerankOptions::strict()
    } else {
        SkeletonRerankOptions::lenient()
    };

    // Convert hits to anchor hits for re-ranking
    let anchor_hits: Vec<QuantumAnchorHit> = payload
        .results
        .iter()
        .map(|hit| QuantumAnchorHit {
            anchor_id: format!("{}#{}", hit.path, hit.stem),
            vector_score: hit.score,
        })
        .collect();

    // Apply skeleton re-ranking
    let validated = skeleton_rerank(anchor_hits, &registry, &topology, &rerank_options);

    // Build XML result
    let xml = render_agentic_nav_result(query, &validated, limit);

    Ok(xml)
}

/// Build page index trees from search hits.
///
/// This function extracts the AST trees for documents found in the search results,
/// enabling skeleton-based validation and re-ranking.
fn build_page_index_trees_from_hits(
    hits: &[crate::link_graph::LinkGraphHit],
    index: &LinkGraphIndex,
) -> std::collections::HashMap<String, Vec<crate::link_graph::PageIndexNode>> {
    let mut trees = std::collections::HashMap::new();

    for hit in hits {
        // Use path as doc_id (it's the relative path like "docs/alpha.md")
        let doc_id = &hit.path;

        // Skip if we already have this tree
        if trees.contains_key(doc_id) {
            continue;
        }

        // Get the page index tree from the index
        if let Some(tree) = index.page_index(doc_id) {
            trees.insert(doc_id.clone(), tree.to_vec());
        }
    }

    trees
}

/// Render agentic navigation result as XML.
fn render_agentic_nav_result(
    query: &str,
    validated: &[crate::link_graph::addressing::SkeletonValidatedHit],
    limit: usize,
) -> String {
    use std::fmt::Write;
    let mut xml = String::new();

    macro_rules! xml_line {
        ($($arg:tt)*) => {
            if writeln!(xml, $($arg)*).is_err() {
                unreachable!("writing XML into String cannot fail");
            }
        };
    }

    macro_rules! xml_inline {
        ($($arg:tt)*) => {
            if write!(xml, $($arg)*).is_err() {
                unreachable!("writing XML into String cannot fail");
            }
        };
    }

    xml_line!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>");
    xml_line!("<agentic_nav_result>");
    xml_line!("  <query>{}</query>", xml_escape(query));
    xml_line!("  <candidates>");

    for v in validated.iter().take(limit) {
        xml_line!("    <candidate>");
        xml_line!("      <doc_id>{}</doc_id>", xml_escape(&v.doc_id));
        xml_line!(
            "      <anchor_id>{}</anchor_id>",
            xml_escape(&v.hit.anchor_id)
        );
        xml_line!("      <is_valid>{}</is_valid>", v.is_valid);
        xml_line!("      <score>{:.4}</score>", v.reranked_score);

        // Add navigation hint based on validation status and structural position
        let hint = generate_navigation_hint(v);
        xml_line!(
            "      <navigation_hint>{}</navigation_hint>",
            xml_escape(&hint)
        );

        if let Some(ref path) = v.structural_path {
            xml_line!("      <structural_path>");
            for segment in path {
                xml_line!("        <segment>{}</segment>", xml_escape(segment));
            }
            xml_line!("      </structural_path>");
        }
        xml_line!("    </candidate>");
    }

    xml_line!("  </candidates>");
    xml_line!("  <total_found>{}</total_found>", validated.len());
    xml_inline!("</agentic_nav_result>");

    xml
}

/// Generate a navigation hint for why this candidate is recommended.
///
/// Based on `OrcaLoca` 2025, agents need to understand "why" a result is suggested.
fn generate_navigation_hint(v: &crate::link_graph::addressing::SkeletonValidatedHit) -> String {
    if !v.is_valid {
        return "Orphaned anchor - content may have changed. Verify before relying on this reference.".to_string();
    }

    let path_depth = v.structural_path.as_ref().map_or(0, std::vec::Vec::len);

    match path_depth {
        0 => "Root-level node - provides high-level overview of the document.".to_string(),
        1 => "Top-level section - good entry point for understanding this topic.".to_string(),
        2..=3 => format!(
            "Nested section at depth {path_depth} - contains specific implementation details."
        ),
        _ => "Deeply nested section - highly specific content, may require parent context."
            .to_string(),
    }
}

/// XML escape helper.
fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
#[path = "../../../tests/unit/zhenfa_router/native/agentic_nav/mod.rs"]
mod tests;
