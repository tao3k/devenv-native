//! Hierarchical `PageIndex` CLI command handler.

use crate::helpers::emit;
use crate::types::Cli;
use anyhow::{Context, Result};
use serde::Serialize;
use serde_json::json;
use xiuxian_wendao::{LinkGraphIndex, LinkGraphMetadata, PageIndexNode};

#[derive(Debug, Clone, Serialize)]
struct PageIndexPayload {
    query: String,
    resolved: LinkGraphMetadata,
    root_count: usize,
    roots: Vec<PageIndexNodeView>,
}

#[derive(Debug, Clone, Serialize)]
struct PageIndexNodeView {
    node_id: String,
    title: String,
    level: usize,
    text: String,
    summary: Option<String>,
    children: Vec<PageIndexNodeView>,
    metadata: PageIndexMetaView,
}

#[derive(Debug, Clone, Serialize)]
struct PageIndexMetaView {
    line_range: (usize, usize),
    token_count: usize,
    is_thinned: bool,
}

impl From<&xiuxian_wendao::PageIndexMeta> for PageIndexMetaView {
    fn from(value: &xiuxian_wendao::PageIndexMeta) -> Self {
        Self {
            line_range: value.line_range,
            token_count: value.token_count,
            is_thinned: value.is_thinned,
        }
    }
}

impl From<&PageIndexNode> for PageIndexNodeView {
    fn from(value: &PageIndexNode) -> Self {
        Self {
            node_id: value.node_id.clone(),
            title: value.title.clone(),
            level: value.level,
            text: value.text.to_string(),
            summary: value.summary.clone(),
            children: value.children.iter().map(Self::from).collect(),
            metadata: PageIndexMetaView::from(&value.metadata),
        }
    }
}

pub(super) fn handle_page_index(
    cli: &Cli,
    index: Option<&LinkGraphIndex>,
    stem: &str,
) -> Result<()> {
    let index = index.context("link_graph index is required for page-index command")?;
    let candidates = index.resolve_metadata_candidates(stem);
    match candidates.len() {
        0 => emit(&Option::<PageIndexPayload>::None, cli.output_or_json()),
        1 => {
            let resolved = candidates
                .into_iter()
                .next()
                .context("page-index candidate unexpectedly missing")?;
            let roots: Vec<PageIndexNodeView> = index
                .page_index(&resolved.path)
                .map(|rows| rows.iter().map(PageIndexNodeView::from).collect())
                .unwrap_or_default();
            emit(
                &PageIndexPayload {
                    query: stem.to_string(),
                    resolved,
                    root_count: roots.len(),
                    roots,
                },
                cli.output_or_json(),
            )
        }
        _ => {
            let payload = json!({
                "error": "ambiguous_stem",
                "query": stem,
                "count": candidates.len(),
                "message": "multiple documents matched this stem/id/path; use full id or path",
                "candidates": candidates,
            });
            emit(&payload, cli.output_or_json())
        }
    }
}
