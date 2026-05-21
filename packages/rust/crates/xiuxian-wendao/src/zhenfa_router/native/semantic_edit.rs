//! `zhenfa_router::native::semantic_edit` owns Wendao zhenfa router native semantic edit behavior.

use std::fmt::Write;

use schemars::JsonSchema;
use serde::Deserialize;
use xiuxian_zhenfa::{ZhenfaContext, ZhenfaError, zhenfa_tool};

use crate::link_graph::{Address, ModificationError, replace_byte_range, resolve_node};
use crate::parsers::markdown::section_create;

use super::context::WendaoContextExt;

/// Arguments for semantic section editing via Triple-A addressing.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct WendaoSemanticEditArgs {
    /// Document stem or ID (e.g., "README" or "docs/architecture").
    doc: String,
    /// Semantic address using Triple-A protocol.
    address: String,
    /// New content for the section.
    new_content: String,
    /// Verify content hash before modification (prevents concurrent edit conflicts).
    #[serde(default)]
    verify_hash: Option<bool>,
    /// Create the section if it doesn't exist (only for path-based addresses).
    #[serde(default)]
    create_if_missing: Option<bool>,
    /// Generate a `:ID: <uuid>` property drawer for newly created sections.
    /// Only applies when `create_if_missing` is true.
    #[serde(default)]
    generate_id: Option<bool>,
    /// Optional prefix for generated IDs (e.g., "arch" -> ":ID: arch-abc123").
    #[serde(default)]
    id_prefix: Option<String>,
}

/// Edit a section in a document using semantic addressing (Triple-A protocol).
///
/// This tool performs atomic byte-range modifications, preserving document formatting
/// and avoiding the "format violence" of full-tree re-rendering.
///
/// The `verify_hash` option enables optimistic concurrency control:
/// the tool will fail if the section's content hash has changed since reading.
///
/// # Errors
///
/// Returns a [`ZhenfaError`] when the address is invalid, the document or section cannot be
/// resolved, the file cannot be read or written, or the byte-range edit fails validation.
#[allow(clippy::needless_pass_by_value)]
#[allow(missing_docs)]
#[zhenfa_tool(
    name = "wendao.semantic_edit",
    description = "Edit a section in a document using semantic addressing with atomic byte-range modification.",
    tool_struct = "WendaoSemanticEditTool",
    mutation_scope = "wendao.semantic_edit"
)]
pub fn wendao_semantic_edit(
    ctx: &ZhenfaContext,
    args: WendaoSemanticEditArgs,
) -> Result<String, ZhenfaError> {
    semantic_edit_impl(ctx, &args)
}

#[allow(clippy::too_many_lines)]
fn semantic_edit_impl(
    ctx: &ZhenfaContext,
    args: &WendaoSemanticEditArgs,
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

    let trees = index.all_page_index_trees();
    let resolved = resolve_node(trees, &address, doc_id);

    // Read document content (needed for both edit and create)
    let doc_path = index.doc_path(&args.doc).ok_or_else(|| {
        ZhenfaError::execution(format!("document path not found: '{}'", args.doc))
    })?;

    let root = index.root();
    let full_path = root.join(doc_path);
    let content = std::fs::read_to_string(&full_path).map_err(|e| {
        ZhenfaError::execution(format!("failed to read document '{doc_path}': {e}"))
    })?;

    // Handle missing section with create_if_missing
    let (new_content, section_title, byte_start, byte_end, new_hash, sibling_context) =
        if let Some(r) = resolved {
            // Existing section - perform atomic modification
            let node = &r.node;
            let (byte_start, byte_end) = node.metadata.byte_range.ok_or_else(|| {
                ZhenfaError::execution(format!(
                    "section '{}' has no byte range information",
                    node.node_id
                ))
            })?;

            let expected_hash = if args.verify_hash.unwrap_or(true) {
                node.metadata.content_hash.as_deref()
            } else {
                None
            };

            let result = replace_byte_range(
                &content,
                byte_start,
                byte_end,
                &args.new_content,
                expected_hash,
            )
            .map_err(|e| match e {
                ModificationError::ByteRangeOutOfBounds {
                    start,
                    end,
                    content_len,
                } => {
                    ZhenfaError::execution(format!(
                        "byte range out of bounds: {start}-{end} (content length: {content_len})"
                    ))
                }
                ModificationError::HashMismatch { expected, actual } => ZhenfaError::execution(
                    format!(
                        "content hash mismatch: expected '{expected}', got '{actual}'. The section may have been modified since you read it."
                    ),
                ),
                ModificationError::NoByteRange => {
                    ZhenfaError::execution("section has no byte range information")
                }
                ModificationError::DeltaOverflow { lhs, rhs } => ZhenfaError::execution(
                    format!("section update length overflow while comparing {lhs} and {rhs}"),
                ),
                ModificationError::RangeAdjustmentOverflow { base, delta } => {
                    ZhenfaError::execution(format!(
                        "section update range adjustment overflow for base {base} with delta {delta}"
                    ))
                }
            })?;

            (
                result.new_content,
                node.title.clone(),
                byte_start,
                byte_end,
                result.new_hash,
                None,
            )
        } else {
            // Section not found - check if create_if_missing is enabled
            if !args.create_if_missing.unwrap_or(false) {
                return Err(ZhenfaError::execution(format!(
                    "address '{}' not found in document '{}'",
                    args.address, args.doc
                )));
            }

            // Only Path-type addresses support create_if_missing
            let path_components = match &address {
                Address::Path(components) => components.clone(),
                Address::Id(_) => {
                    return Err(ZhenfaError::invalid_arguments(
                        "create_if_missing requires a path-based address (e.g., /Section/Subsection). ID-based addresses cannot be auto-created.",
                    ));
                }
                Address::Hash(_) => {
                    return Err(ZhenfaError::invalid_arguments(
                        "create_if_missing requires a path-based address (e.g., /Section/Subsection). Hash-based addresses cannot be auto-created.",
                    ));
                }
                Address::Block { .. } => {
                    return Err(ZhenfaError::invalid_arguments(
                        "create_if_missing requires a path-based address (e.g., /Section/Subsection). Block-based addresses cannot be auto-created.",
                    ));
                }
            };

            // Find insertion point
            let insertion_info = section_create::find_insertion_point(&content, &path_components);

            // Build new section content with optional ID generation
            let build_options = section_create::BuildSectionOptions {
                generate_id: args.generate_id.unwrap_or(false),
                id_prefix: args.id_prefix.clone(),
            };

            let sections_content = section_create::build_new_sections_content_with_options(
                &insertion_info.remaining_path,
                insertion_info.start_level,
                &args.new_content,
                &build_options,
            );

            // Build sibling context string for response
            let sibling_context = format_sibling_context(&insertion_info);

            // Insert at determined position
            let mut new_doc = String::with_capacity(content.len() + sections_content.len());
            new_doc.push_str(&content[..insertion_info.insertion_byte]);
            if insertion_info.insertion_byte > 0
                && !content[..insertion_info.insertion_byte].ends_with('\n')
            {
                new_doc.push('\n');
            }
            new_doc.push_str(&sections_content);
            new_doc.push_str(&content[insertion_info.insertion_byte..]);

            let new_hash = section_create::compute_content_hash(&sections_content);
            let section_title = path_components
                .last()
                .cloned()
                .unwrap_or_else(|| "Section".to_string());

            let sibling_ctx: Option<String> = Some(sibling_context);
            (
                new_doc,
                section_title,
                insertion_info.insertion_byte,
                insertion_info.insertion_byte + sections_content.len(),
                new_hash,
                sibling_ctx,
            )
        };

    // Write back to file
    std::fs::write(&full_path, &new_content).map_err(|e| {
        ZhenfaError::execution(format!("failed to write document '{doc_path}': {e}"))
    })?;

    // Build response
    let sibling_xml: String = sibling_context.unwrap_or_default();
    Ok(format!(
        "<edit_result>\n\
         \x20  <document>{}</document>\n\
         \x20  <section title=\"{}\"/>\n\
         \x20  <address original=\"{}\"/>\n\
         \x20  <byte_range start=\"{}\" end=\"{}\"/>\n\
         \x20  <new_hash>{}</new_hash>{}\n\
         </edit_result>\n",
        args.doc, section_title, args.address, byte_start, byte_end, new_hash, sibling_xml
    ))
}

/// Format sibling context as XML for the response.
fn format_sibling_context(info: &section_create::InsertionInfo) -> String {
    let mut result = String::new();

    if let Some(ref prev) = info.prev_sibling {
        let preview = if prev.preview.is_empty() {
            "..."
        } else {
            &prev.preview
        };
        let _ = write!(
            result,
            "\n         \x20  <prev_sibling title=\"{}\">{}</prev_sibling>",
            prev.title, preview
        );
    }

    if let Some(ref next) = info.next_sibling {
        let preview = if next.preview.is_empty() {
            "..."
        } else {
            &next.preview
        };
        let _ = write!(
            result,
            "\n         \x20  <next_sibling title=\"{}\">{}</next_sibling>",
            next.title, preview
        );
    }

    result
}

#[cfg(test)]
#[path = "../../../tests/unit/zhenfa_router/native/semantic_edit.rs"]
mod tests;
