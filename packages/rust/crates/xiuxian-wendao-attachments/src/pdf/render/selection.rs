//! Page selection policy for PDF render shard planning.

use std::path::Path;

use lopdf::Document as LopdfDocument;

use crate::pdf::source_range::source_page_range_all_page_indices;

use super::types::{PdfPageRenderSelection, PdfRenderRoutingDecision};

pub(super) fn resolve_source_page_range_selection(
    path: &Path,
    selection: PdfPageRenderSelection,
) -> Result<(u32, RenderPageSelection), String> {
    let page_count = source_pdf_page_count(path)?;
    match selection {
        PdfPageRenderSelection::AllPages => Ok((page_count, RenderPageSelection::All)),
        PdfPageRenderSelection::RegionShards => {
            Err("region_shards selection requires configured region requests".to_string())
        }
        PdfPageRenderSelection::ShardFallbackPages => {
            if page_count == 0 {
                return Ok((
                    page_count,
                    RenderPageSelection::Skip {
                        page_count,
                        routing_decision: PdfRenderRoutingDecision::FullDoclingFallback,
                        reason: "source PDF page tree is empty".to_string(),
                    },
                ));
            }
            Ok((
                page_count,
                RenderPageSelection::Selected(source_page_range_all_page_indices(page_count)),
            ))
        }
    }
}

/// Return the number of pages in a source PDF using the lightweight page tree.
///
/// # Errors
///
/// Returns an error when the PDF cannot be loaded or the page count exceeds
/// the public `u32` contract.
pub fn source_pdf_page_count(path: &Path) -> Result<u32, String> {
    let document = LopdfDocument::load(path)
        .map_err(|error| format!("load PDF page tree with lopdf: {error}"))?;
    u32::try_from(document.get_pages().len())
        .map_err(|_| "PDF page count exceeds u32 range".to_string())
}
pub(super) enum RenderPageSelection {
    All,
    Selected(Vec<i32>),
    Skip {
        page_count: u32,
        routing_decision: PdfRenderRoutingDecision,
        reason: String,
    },
}

impl RenderPageSelection {
    pub(super) fn selected_page_indices(&self) -> Option<&[i32]> {
        match self {
            Self::Selected(page_indices) => Some(page_indices.as_slice()),
            Self::All | Self::Skip { .. } => None,
        }
    }
}

#[cfg(feature = "pdf-render")]
pub(super) fn resolve_page_selection(
    path: &Path,
    selection: PdfPageRenderSelection,
) -> Result<RenderPageSelection, String> {
    match selection {
        PdfPageRenderSelection::AllPages => Ok(RenderPageSelection::All),
        PdfPageRenderSelection::ShardFallbackPages => {
            Ok(resolve_shard_fallback_page_selection(path))
        }
        PdfPageRenderSelection::RegionShards => {
            Err("region_shards selection requires configured region requests".to_string())
        }
    }
}

#[cfg(feature = "pdf-render")]
fn resolve_shard_fallback_page_selection(_path: &Path) -> RenderPageSelection {
    RenderPageSelection::All
}
