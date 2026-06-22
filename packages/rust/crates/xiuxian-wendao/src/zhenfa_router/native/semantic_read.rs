//! `zhenfa_router::native::semantic_read` owns Wendao zhenfa router native semantic read behavior.

use std::fmt::Write;

use schemars::JsonSchema;
use serde::Deserialize;
use xiuxian_zhenfa::{ZhenfaContext, ZhenfaError};

use crate::link_graph::{
    Address, MatchType, PageIndexNode, ResolveMode, resolve_node, resolve_with_indices,
};

use super::context::WendaoContextExt;

/// Arguments for semantic section reading via Triple-A addressing.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct WendaoSemanticReadArgs {
    /// Document stem or ID (e.g., "README" or "docs/architecture").
    doc: String,
    /// Semantic address using Triple-A protocol:
    /// - `#id` - Explicit anchor ID (e.g., `#arch-v1`)
    /// - `/path/to/heading` - Structural path through heading hierarchy
    /// - `@hash` - Content hash for self-healing lookup
    address: String,
    /// Include surrounding context (parent section content).
    #[serde(default)]
    include_context: Option<bool>,
    /// Enable fuzzy path matching (allows path drift tolerance).
    #[serde(default)]
    fuzzy: Option<bool>,
}

/// Read a section from a document using semantic addressing (Triple-A protocol).
///
/// The address parameter supports three formats:
/// - `#anchor-id` - Resolve by explicit `:ID:` property drawer attribute
/// - `/Heading/Subheading` - Resolve by structural heading path
/// - `@content-hash` - Resolve by Blake3 content fingerprint
///
/// Resolution follows the Triple-A protocol: ID → Path → Hash fallback.
/// When `fuzzy` is enabled, path drift tolerance allows approximate matches.
///
/// # Errors
///
/// Returns a [`ZhenfaError`] when the address is invalid, the document cannot be resolved or
/// read, or the requested section lacks the metadata required for byte-precise extraction.
#[allow(clippy::needless_pass_by_value, clippy::too_many_lines)]
#[allow(missing_docs)]
pub fn wendao_semantic_read(
    ctx: &ZhenfaContext,
    args: WendaoSemanticReadArgs,
) -> Result<String, ZhenfaError> {
    let address = Address::parse(&args.address).ok_or_else(|| {
        ZhenfaError::invalid_arguments(format!(
            "invalid address format: '{}'. Use #id, /path/to/heading, or @hash",
            args.address
        ))
    })?;

    let index = ctx.link_graph_index()?;
    let doc_id = index.resolve_doc_id_pub(&args.doc).ok_or_else(|| {
        ZhenfaError::invalid_arguments(format!("document not found: '{}'", args.doc))
    })?;

    // Build dual indices for enhanced resolution
    let registry = index.build_registry_index();
    let topology = index.build_topology_index();

    // Determine resolution mode based on fuzzy flag
    let mode = if args.fuzzy.unwrap_or(false) {
        ResolveMode::Discover {
            fuzzy: true,
            max_results: 5,
        }
    } else {
        ResolveMode::Anchor
    };

    // Try enhanced resolution first
    let (node, resolved_path, resolved_id, match_type, similarity) =
        if let Ok(enhanced) = resolve_with_indices(&registry, &topology, &address, doc_id, mode) {
            (
                enhanced.node,
                enhanced.resolved_path,
                enhanced.resolved_id,
                enhanced.match_type,
                enhanced.similarity,
            )
        } else {
            // Fallback to legacy resolution
            let trees = index.all_page_index_trees();
            let resolved = resolve_node(trees, &address, doc_id).ok_or_else(|| {
                ZhenfaError::execution(format!(
                    "address '{}' not found in document '{}'",
                    args.address, args.doc
                ))
            })?;
            let path = resolved.node.metadata.structural_path.clone();
            let id = resolved.node.metadata.attributes.get("ID").cloned();
            (resolved.node, path, id, MatchType::Exact, 1.0)
        };

    // Read document content via index root
    let doc_path = index.doc_path(&args.doc).ok_or_else(|| {
        ZhenfaError::execution(format!("document path not found: '{}'", args.doc))
    })?;
    let root = index.root();
    let full_path = root.join(doc_path);
    let content = std::fs::read_to_string(&full_path).map_err(|e| {
        ZhenfaError::execution(format!("failed to read document '{doc_path}': {e}"))
    })?;

    // Extract section content using byte range
    let (byte_start, byte_end) = node.metadata.byte_range.ok_or_else(|| {
        ZhenfaError::execution(format!(
            "section '{}' has no byte range information",
            node.node_id
        ))
    })?;

    let section_content = &content[byte_start..byte_end];
    let include_context = args.include_context.unwrap_or(false);

    // Build enhanced response with resolution metadata
    let mut response = String::new();
    let _ = writeln!(
        response,
        "<section node_id=\"{}\" title=\"{}\" address=\"{}\">",
        node.node_id,
        node.title,
        address.to_display_string()
    );

    // Add resolution metadata
    response.push_str("  <resolution>\n");
    let _ = writeln!(
        response,
        "    <resolved_path>{}</resolved_path>",
        resolved_path.join("/")
    );
    if let Some(ref id) = resolved_id {
        let _ = writeln!(response, "    <resolved_id>#{id}</resolved_id>");
    }
    let _ = writeln!(
        response,
        "    <match_type>{}</match_type>",
        match_type_to_string(match_type)
    );
    let _ = writeln!(response, "    <similarity>{similarity:.2}</similarity>");
    response.push_str("  </resolution>\n");

    let _ = writeln!(
        response,
        "  <metadata line_range=\"{}-{}\" byte_range=\"{}-{}\" token_count=\"{}\"/>",
        node.metadata.line_range.0,
        node.metadata.line_range.1,
        byte_start,
        byte_end,
        node.metadata.token_count
    );

    if let Some(ref hash) = node.metadata.content_hash {
        let _ = writeln!(response, "  <content_hash>{hash}</content_hash>");
    }

    if include_context {
        // Include parent context
        if let Some(parent_id) = &node.parent_id {
            let trees = index.all_page_index_trees();
            if let Some(parent_nodes) = trees.get(doc_id)
                && let Some(parent) = find_node_by_id(parent_nodes, parent_id)
                && let Some((p_start, p_end)) = parent.metadata.byte_range
            {
                let parent_content = &content[p_start..p_end];
                response.push_str("  <parent_context>\n");
                let _ = writeln!(response, "    <title>{}</title>", parent.title);
                response.push_str("    <content><![CDATA[\n");
                response.push_str(parent_content);
                response.push_str("\n    ]]></content>\n");
                response.push_str("  </parent_context>\n");
            }
        }
    }

    response.push_str("  <content><![CDATA[\n");
    response.push_str(section_content);
    response.push_str("\n  ]]></content>\n");
    response.push_str("</section>\n");

    Ok(response)
}

/// Convert `MatchType` to human-readable string.
fn match_type_to_string(match_type: MatchType) -> &'static str {
    match match_type {
        MatchType::Exact => "exact",
        MatchType::Suffix => "suffix",
        MatchType::TitleSubstring => "title_substring",
        MatchType::TitleFuzzy => "title_fuzzy",
        MatchType::HashFallback => "hash_fallback",
        MatchType::CaseInsensitive => "case_insensitive",
    }
}

fn find_node_by_id(nodes: &[PageIndexNode], target_id: &str) -> Option<PageIndexNode> {
    for node in nodes {
        if node.node_id == target_id {
            return Some(node.clone());
        }
        if let Some(found) = find_node_by_id(&node.children, target_id) {
            return Some(found);
        }
    }
    None
}

#[cfg(test)]
#[path = "../../../tests/unit/zhenfa_router/native/semantic_read.rs"]
mod tests;
