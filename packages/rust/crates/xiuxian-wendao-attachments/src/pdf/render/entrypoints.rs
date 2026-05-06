//! Public render shard entrypoints and JSON input decoding.

use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use super::batches::{build_shard_manifest_batch, write_shard_artifact_batches};
use super::document::source_page_range_document_manifests;
#[cfg(feature = "pdf-render")]
use super::document::{bind_pdfium, render_document_manifests, render_document_region_manifests};
use super::identity::{checked_len_u32, is_pdf_path, sha256_hex, source_page_range_profile};
use super::report::{RenderShardContext, ReportParts};
#[cfg(feature = "pdf-render")]
use super::selection::resolve_page_selection;
use super::selection::{RenderPageSelection, resolve_source_page_range_selection};
#[cfg(feature = "pdf-render")]
use super::types::{PdfPageRegionRenderRequest, PdfRenderRoutingDecision};
use super::types::{PdfPageRenderProfile, PdfPageRenderSelection, PdfPageRenderShardReport};

/// # Errors
///
/// Returns an error if the path cannot be read or Arrow report files cannot be
/// written. Missing `PDFium` libraries are represented as fallback reports rather
/// than errors.
#[cfg(feature = "pdf-render")]
pub fn render_pdf_page_shards(
    path: &Path,
    output_dir: &Path,
    profile: &PdfPageRenderProfile,
) -> Result<PdfPageRenderShardReport, String> {
    render_pdf_page_shards_with_selection(
        path,
        output_dir,
        profile,
        PdfPageRenderSelection::AllPages,
    )
}

/// # Errors
///
/// Returns an error if the path cannot be read or Arrow report files cannot be
/// written. Missing `PDFium` libraries are represented as fallback reports rather
/// than errors.
#[cfg(feature = "pdf-render")]
pub fn render_pdf_page_shards_with_selection(
    path: &Path,
    output_dir: &Path,
    profile: &PdfPageRenderProfile,
    selection: PdfPageRenderSelection,
) -> Result<PdfPageRenderShardReport, String> {
    let context = RenderShardContext::new(path, output_dir, profile, selection);
    if !is_pdf_path(path) {
        return Ok(context.report(ReportParts::unsupported("unsupported non-PDF input")));
    }

    let page_selection = match resolve_page_selection(path, selection) {
        Ok(page_selection) => page_selection,
        Err(error) => {
            return Ok(context.report(ReportParts::preflight_failed(format!(
                "analyze PDF `{}` for render selection: {error}",
                path.display()
            ))));
        }
    };
    if let RenderPageSelection::Skip {
        page_count,
        routing_decision,
        reason,
    } = &page_selection
    {
        return Ok(context.report(ReportParts::skipped(
            *page_count,
            *routing_decision,
            reason.clone(),
        )));
    }

    let source_bytes =
        fs::read(path).map_err(|error| format!("read PDF `{}`: {error}", path.display()))?;
    let source_hash = sha256_hex(&source_bytes);
    let pdfium = match bind_pdfium() {
        Ok(pdfium) => pdfium,
        Err(error) => return Ok(context.report(ReportParts::fallback(0, 0, error))),
    };

    let document = match pdfium.load_pdf_from_file(path, None) {
        Ok(document) => document,
        Err(error) => {
            return Ok(context.report(ReportParts::preflight_failed(format!(
                "load PDF `{}`: {error}",
                path.display()
            ))));
        }
    };

    let page_count = u32::try_from(document.pages().len()).unwrap_or_default();
    let manifests = match render_document_manifests(
        &document,
        &context,
        &source_hash,
        page_selection.selected_page_indices(),
    ) {
        Ok(manifests) => manifests,
        Err(fallback) => return Ok(context.report(fallback)),
    };

    let manifest_batch = build_shard_manifest_batch(&manifests)?;
    let (manifest_arrow_path, ocr_input_arrow_path, pending_resource_arrow_path) =
        write_shard_artifact_batches(output_dir, manifests.as_slice(), manifest_batch)?;

    Ok(context.report(ReportParts::rendered(
        page_count,
        checked_len_u32(manifests.len()),
        manifest_arrow_path,
        ocr_input_arrow_path,
        pending_resource_arrow_path,
    )))
}

/// # Errors
///
/// Returns an error if the path cannot be read, selected page indices cannot be
/// converted, or Arrow report files cannot be written. Missing `PDFium`
/// libraries are represented as fallback reports rather than errors.
#[cfg(feature = "pdf-render")]
pub fn render_pdf_page_shards_for_page_indices(
    path: &Path,
    output_dir: &Path,
    profile: &PdfPageRenderProfile,
    page_indices: &[u32],
) -> Result<PdfPageRenderShardReport, String> {
    let context =
        RenderShardContext::new(path, output_dir, profile, PdfPageRenderSelection::AllPages);
    if !is_pdf_path(path) {
        return Ok(context.report(ReportParts::unsupported("unsupported non-PDF input")));
    }
    let selected_page_indices = page_indices
        .iter()
        .map(|page_index| {
            i32::try_from(*page_index)
                .map_err(|_| format!("selected page index exceeds i32: {page_index}"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    if selected_page_indices.is_empty() {
        return Ok(context.report(ReportParts::skipped(
            0,
            PdfRenderRoutingDecision::HybridPageOcrCandidate,
            "no selected page shards requested".to_string(),
        )));
    }

    let source_bytes =
        fs::read(path).map_err(|error| format!("read PDF `{}`: {error}", path.display()))?;
    let source_hash = sha256_hex(&source_bytes);
    let pdfium = match bind_pdfium() {
        Ok(pdfium) => pdfium,
        Err(error) => return Ok(context.report(ReportParts::fallback(0, 0, error))),
    };

    let document = match pdfium.load_pdf_from_file(path, None) {
        Ok(document) => document,
        Err(error) => {
            return Ok(context.report(ReportParts::preflight_failed(format!(
                "load PDF `{}`: {error}",
                path.display()
            ))));
        }
    };
    let page_count = u32::try_from(document.pages().len()).unwrap_or_default();
    let manifests = match render_document_manifests(
        &document,
        &context,
        &source_hash,
        Some(selected_page_indices.as_slice()),
    ) {
        Ok(manifests) => manifests,
        Err(fallback) => return Ok(context.report(fallback)),
    };

    let manifest_batch = build_shard_manifest_batch(&manifests)?;
    let (manifest_arrow_path, ocr_input_arrow_path, pending_resource_arrow_path) =
        write_shard_artifact_batches(output_dir, manifests.as_slice(), manifest_batch)?;

    Ok(context.report(ReportParts::rendered(
        page_count,
        checked_len_u32(manifests.len()),
        manifest_arrow_path,
        ocr_input_arrow_path,
        pending_resource_arrow_path,
    )))
}

/// # Errors
///
/// Returns an error if the path cannot be read or Arrow report files cannot be
/// written. Missing `PDFium` libraries are represented as fallback reports rather
/// than errors.
pub fn prepare_pdf_source_page_range_ocr_shards_with_selection(
    path: &Path,
    output_dir: &Path,
    profile: &PdfPageRenderProfile,
    selection: PdfPageRenderSelection,
) -> Result<PdfPageRenderShardReport, String> {
    let source_profile = source_page_range_profile(profile);
    let context = RenderShardContext::new(path, output_dir, &source_profile, selection);
    if !is_pdf_path(path) {
        return Ok(context.report(ReportParts::unsupported("unsupported non-PDF input")));
    }

    let source_bytes =
        fs::read(path).map_err(|error| format!("read PDF `{}`: {error}", path.display()))?;
    let source_hash = sha256_hex(&source_bytes);
    let (page_count, page_selection) = match resolve_source_page_range_selection(path, selection) {
        Ok(selection) => selection,
        Err(error) => {
            return Ok(context.report(ReportParts::preflight_failed(format!(
                "analyze PDF `{}` for source page range selection: {error}",
                path.display()
            ))));
        }
    };
    if let RenderPageSelection::Skip {
        page_count,
        routing_decision,
        reason,
    } = &page_selection
    {
        return Ok(context.report(ReportParts::skipped(
            *page_count,
            *routing_decision,
            reason.clone(),
        )));
    }

    let manifests = match source_page_range_document_manifests(
        &context,
        &source_hash,
        page_count,
        page_selection.selected_page_indices(),
    ) {
        Ok(manifests) => manifests,
        Err(fallback) => return Ok(context.report(fallback)),
    };

    let manifest_batch = build_shard_manifest_batch(&manifests)?;
    let (manifest_arrow_path, ocr_input_arrow_path, pending_resource_arrow_path) =
        write_shard_artifact_batches(output_dir, manifests.as_slice(), manifest_batch)?;

    Ok(context.report(ReportParts::rendered(
        page_count,
        checked_len_u32(manifests.len()),
        manifest_arrow_path,
        ocr_input_arrow_path,
        pending_resource_arrow_path,
    )))
}
/// # Errors
///
/// Returns an error if the PDF cannot be read, the requested regions cannot be
/// rendered, or Arrow artifact files cannot be written. Missing `PDFium`
/// libraries are represented as fallback reports rather than errors.
#[cfg(feature = "pdf-render")]
pub fn render_pdf_region_shards(
    path: &Path,
    output_dir: &Path,
    profile: &PdfPageRenderProfile,
    regions: &[PdfPageRegionRenderRequest],
) -> Result<PdfPageRenderShardReport, String> {
    let context = RenderShardContext::new(
        path,
        output_dir,
        profile,
        PdfPageRenderSelection::RegionShards,
    );
    if !is_pdf_path(path) {
        return Ok(context.report(ReportParts::unsupported("unsupported non-PDF input")));
    }
    if regions.is_empty() {
        return Ok(context.report(ReportParts::skipped(
            0,
            PdfRenderRoutingDecision::HybridPageOcrCandidate,
            "no region shards requested".to_string(),
        )));
    }

    let source_bytes =
        fs::read(path).map_err(|error| format!("read PDF `{}`: {error}", path.display()))?;
    let source_hash = sha256_hex(&source_bytes);
    let pdfium = match bind_pdfium() {
        Ok(pdfium) => pdfium,
        Err(error) => return Ok(context.report(ReportParts::fallback(0, 0, error))),
    };
    let document = match pdfium.load_pdf_from_file(path, None) {
        Ok(document) => document,
        Err(error) => {
            return Ok(context.report(ReportParts::preflight_failed(format!(
                "load PDF `{}`: {error}",
                path.display()
            ))));
        }
    };

    let page_count = u32::try_from(document.pages().len()).unwrap_or_default();
    let manifests =
        match render_document_region_manifests(&document, &context, &source_hash, regions) {
            Ok(manifests) => manifests,
            Err(fallback) => return Ok(context.report(fallback)),
        };
    let manifest_batch = build_shard_manifest_batch(&manifests)?;
    let (manifest_arrow_path, ocr_input_arrow_path, pending_resource_arrow_path) =
        write_shard_artifact_batches(output_dir, manifests.as_slice(), manifest_batch)?;

    Ok(context.report(ReportParts::rendered(
        page_count,
        checked_len_u32(manifests.len()),
        manifest_arrow_path,
        ocr_input_arrow_path,
        pending_resource_arrow_path,
    )))
}

/// # Errors
///
/// Returns an error if the input JSON does not decode to audit paths.
pub fn read_render_paths_from_json(json: &str) -> Result<Vec<PathBuf>, String> {
    #[derive(Deserialize)]
    struct Input {
        source: PathBuf,
    }

    serde_json::from_str::<Vec<Input>>(json)
        .map_err(|error| format!("parse PDF render shard input JSON: {error}"))
        .map(|inputs| inputs.into_iter().map(|input| input.source).collect())
}
