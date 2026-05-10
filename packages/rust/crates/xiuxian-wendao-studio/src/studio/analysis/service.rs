//! Owns the Studio studio analysis service surface.

use std::path::Path;

use crate::studio::StudioState;
use crate::studio::analysis::markdown::{
    CompiledDocument, build_markdown_document_metadata, compile_markdown_ir,
};
use crate::studio::analysis::projection;
use crate::studio::pathing::{normalize_path_like, studio_display_path};
use crate::studio::types::AnalysisNode;
use crate::studio::types::MarkdownAnalysisResponse;
use crate::studio::vfs::resolve_vfs_file_path;

/// Error raised while compiling Studio analysis responses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnalysisError {
    UnsupportedContentType(String),
    Vfs(String),
}

impl std::fmt::Display for AnalysisError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedContentType(content_type) => {
                write!(f, "unsupported content type: {content_type}")
            }
            Self::Vfs(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for AnalysisError {}

pub(crate) fn compile_markdown_nodes(path: &str, content: &str) -> Vec<AnalysisNode> {
    compile_markdown_ir(path, content).nodes
}

pub(crate) async fn analyze_markdown(
    state: &StudioState,
    path: &str,
) -> Result<MarkdownAnalysisResponse, AnalysisError> {
    let normalized_path = normalize_path_like(path).unwrap_or_else(|| path.trim().to_string());
    if !is_markdown_path(path) {
        return Err(AnalysisError::UnsupportedContentType(
            infer_content_type(path).to_string(),
        ));
    }
    if studio_display_path(state, path) != normalized_path {
        return Err(AnalysisError::Vfs(format!(
            "VFS path not found: {}",
            path.trim()
        )));
    }

    let full_path = resolve_vfs_file_path(state, path).map_err(|error| {
        AnalysisError::Vfs(
            error
                .error
                .details
                .clone()
                .unwrap_or_else(|| error.error.message.clone()),
        )
    })?;

    let content = std::fs::read_to_string(&full_path)
        .map_err(|e| AnalysisError::Vfs(format!("Failed to read file: {e}")))?;

    let compiled: CompiledDocument = compile_markdown_ir(path, &content);

    let graph_index_guard = state
        .graph_index
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let graph_index = graph_index_guard
        .as_ref()
        .map(|entry| std::sync::Arc::as_ref(&entry.index));
    let document_metadata = build_markdown_document_metadata(state, path, &content, graph_index);

    let projections = projection::build_mermaid_projections(&compiled.nodes, &compiled.edges);

    Ok(MarkdownAnalysisResponse {
        path: path.to_string(),
        document_hash: compiled.document_hash,
        node_count: compiled.nodes.len(),
        edge_count: compiled.edges.len(),
        nodes: compiled.nodes,
        edges: compiled.edges,
        projections,
        retrieval_atoms: compiled.retrieval_atoms,
        document_metadata: Some(document_metadata),
        diagnostics: compiled.diagnostics,
    })
}

fn is_markdown_path(path: &str) -> bool {
    has_extension(path, &["md", "markdown"])
}

fn infer_content_type(path: &str) -> &'static str {
    if has_extension(path, &["rs"]) {
        "text/x-rust"
    } else if is_markdown_path(path) {
        "text/markdown"
    } else {
        "application/octet-stream"
    }
}

fn has_extension(path: &str, extensions: &[&str]) -> bool {
    Path::new(path)
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| {
            extensions
                .iter()
                .any(|candidate| ext.eq_ignore_ascii_case(candidate))
        })
}
