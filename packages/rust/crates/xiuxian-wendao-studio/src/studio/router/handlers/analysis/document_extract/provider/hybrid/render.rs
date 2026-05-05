use std::path::{Path, PathBuf};

#[cfg(feature = "document-extract-pdf-render")]
use xiuxian_wendao_attachments::pdf::render::render_pdf_region_shards;
use xiuxian_wendao_attachments::pdf::render::{
    PdfPageRegionRenderRequest, PdfPageRenderProfile, PdfPageRenderSelection,
    PdfPageRenderShardReport, PdfRenderRoutingDecision, PdfRenderStatus,
    prepare_pdf_source_page_range_ocr_shards_with_selection,
};
use xiuxian_wendao_server::transport::DocumentExtractFlightRequest;

use super::types::{
    DOCUMENT_EXTRACT_PDF_RENDER_REGIONS_ENV, DOCUMENT_EXTRACT_PDF_RENDER_SELECTION_ENV,
    HybridPdfRegionInput,
};
use crate::studio::router::handlers::analysis::document_extract::registry::default_output_dir;

pub(crate) async fn render_hybrid_page_ocr_shards(
    source: &Path,
    output: &Path,
) -> Result<PdfPageRenderShardReport, String> {
    let selection = hybrid_page_ocr_render_selection();
    let regions = if selection == PdfPageRenderSelection::RegionShards {
        Some(hybrid_page_ocr_region_requests_for_source(source)?)
    } else {
        None
    };
    let source_for_render = source.to_path_buf();
    let output_for_render = output.to_path_buf();
    tokio::task::spawn_blocking(move || {
        if let Some(regions) = regions {
            #[cfg(feature = "document-extract-pdf-render")]
            return render_pdf_region_shards(
                source_for_render.as_path(),
                output_for_render.as_path(),
                &PdfPageRenderProfile::ocr_default(),
                regions.as_slice(),
            );
            #[cfg(not(feature = "document-extract-pdf-render"))]
            let _ = regions;
            #[cfg(not(feature = "document-extract-pdf-render"))]
            return Err(format!(
                "hybrid PDF region shards for `{}` require the `document-extract-pdf-render` feature",
                source_for_render.display()
            ));
        }
        prepare_pdf_source_page_range_ocr_shards_with_selection(
            source_for_render.as_path(),
            output_for_render.as_path(),
            &PdfPageRenderProfile::ocr_default(),
            selection,
        )
    })
    .await
    .map_err(|error| format!("join hybrid PDF OCR render task: {error}"))?
}

fn hybrid_page_ocr_render_selection() -> PdfPageRenderSelection {
    hybrid_page_ocr_render_selection_with_lookup(&|key| std::env::var(key).ok())
}

pub(crate) fn hybrid_page_ocr_render_selection_with_lookup(
    lookup: &dyn Fn(&str) -> Option<String>,
) -> PdfPageRenderSelection {
    match lookup(DOCUMENT_EXTRACT_PDF_RENDER_SELECTION_ENV)
        .unwrap_or_default()
        .trim()
        .replace('-', "_")
        .as_str()
    {
        "all_pages" => PdfPageRenderSelection::AllPages,
        "region_shards" => PdfPageRenderSelection::RegionShards,
        _ => PdfPageRenderSelection::ShardFallbackPages,
    }
}

fn hybrid_page_ocr_region_requests_for_source(
    source: &Path,
) -> Result<Vec<PdfPageRegionRenderRequest>, String> {
    hybrid_page_ocr_region_requests_for_source_with_lookup(source, &|key| std::env::var(key).ok())
}

pub(crate) fn hybrid_page_ocr_region_requests_for_source_with_lookup(
    source: &Path,
    lookup: &dyn Fn(&str) -> Option<String>,
) -> Result<Vec<PdfPageRegionRenderRequest>, String> {
    let regions_json = lookup(DOCUMENT_EXTRACT_PDF_RENDER_REGIONS_ENV).ok_or_else(|| {
        format!("{DOCUMENT_EXTRACT_PDF_RENDER_REGIONS_ENV} is required for region_shards")
    })?;
    let region_inputs = serde_json::from_str::<Vec<HybridPdfRegionInput>>(regions_json.as_str())
        .map_err(|error| format!("parse hybrid PDF region JSON: {error}"))?;
    let mut matching_regions = None;
    for input in region_inputs {
        if paths_match(source, input.source.as_path()) {
            if input.regions.is_empty() {
                return Err(format!(
                    "hybrid PDF region fixture has no regions for `{}`",
                    input.source.display()
                ));
            }
            if matching_regions.replace(input.regions).is_some() {
                return Err(format!(
                    "duplicate hybrid PDF region fixture for `{}`",
                    source.display()
                ));
            }
        }
    }
    matching_regions.ok_or_else(|| {
        format!(
            "no hybrid PDF region fixture matched source `{}`",
            source.display()
        )
    })
}

fn paths_match(left: &Path, right: &Path) -> bool {
    left == right
        || match (left.canonicalize(), right.canonicalize()) {
            (Ok(left), Ok(right)) => left == right,
            _ => false,
        }
}

pub(crate) fn hybrid_page_ocr_request_paths(
    request: &DocumentExtractFlightRequest,
) -> (PathBuf, PathBuf) {
    let source = PathBuf::from(request.source_path.as_str());
    let output = if request.output_dir.trim().is_empty() {
        default_output_dir(source.as_path())
    } else {
        PathBuf::from(request.output_dir.as_str())
    };
    (source, output)
}

pub(crate) fn hybrid_page_ocr_input_arrow_path(
    report: &PdfPageRenderShardReport,
) -> Result<PathBuf, String> {
    if report.status != PdfRenderStatus::Rendered.as_str() {
        return Err(format!(
            "render status `{}` is not eligible for hybrid OCR",
            report.status
        ));
    }
    if report.routing_decision != PdfRenderRoutingDecision::HybridPageOcrCandidate.as_str() {
        return Err(format!(
            "routing decision `{}` is not eligible for hybrid OCR",
            report.routing_decision
        ));
    }
    if report.page_count == 0 {
        return Err("hybrid OCR render report has no pages".to_string());
    }
    report
        .ocr_input_arrow_path
        .as_ref()
        .filter(|path| !path.trim().is_empty())
        .map(PathBuf::from)
        .ok_or_else(|| "hybrid OCR render report is missing `_ocr_input.arrow`".to_string())
}
