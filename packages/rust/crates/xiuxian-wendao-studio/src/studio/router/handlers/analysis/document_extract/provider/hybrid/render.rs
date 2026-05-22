use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use xiuxian_wendao_attachments::pdf::profile::{
    PdfSourcePageProfile, source_pdf_page_profiles_cached,
};
#[cfg(feature = "document-extract-pdf-render")]
use xiuxian_wendao_attachments::pdf::render::render_pdf_page_shards_with_selection;
use xiuxian_wendao_attachments::pdf::render::{
    PdfPageBox, PdfPageRegionRenderRequest, PdfPageRenderProfile, PdfPageRenderSelection,
    PdfPageRenderShardReport, PdfRenderRoutingDecision, PdfRenderStatus,
    prepare_pdf_source_page_range_ocr_shards_with_selection,
};
use xiuxian_wendao_server::transport::DocumentExtractFlightRequest;

use super::profile::{hybrid_page_ocr_profile_planner, is_hosted_vlm_topup_page};
use super::types::{
    DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_MAX_SLICES_ENV,
    DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_PLANNER_ENV,
    DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_TARGET_PIXELS_ENV,
    DOCUMENT_EXTRACT_PDF_HOSTED_VLM_RENDER_DPI_ENV, DOCUMENT_EXTRACT_PDF_REGION_CONTEXT_RATIO_ENV,
    DOCUMENT_EXTRACT_PDF_RENDER_REGIONS_ENV, DOCUMENT_EXTRACT_PDF_RENDER_SELECTION_ENV,
    HybridPdfRegionInput,
};
use crate::studio::router::handlers::analysis::document_extract::registry::default_output_dir;
use xiuxian_wendao_attachments::pdf::ocr::{PdfOcrShardInput, is_hosted_vlm_direct_profile};

const DEFAULT_OCR2_REGION_CONTEXT_RATIO: f64 = 0.18;
const OCR2_REGION_PLANNER_PROFILE_RISK_WINDOW: &str = "profile-risk-window";
const OCR2_REGION_PLANNER_PROFILE_RISK_WINDOW_SLICES: &str = "profile-risk-window-slices";
const OCR2_REGION_PLANNER_PROFILE_RISK_WINDOW_ADAPTIVE: &str = "profile-risk-window-adaptive";
const OCR2_AUTO_REGION_LEFT_RATIO: f64 = 0.18;
const OCR2_AUTO_REGION_RIGHT_RATIO: f64 = 0.82;
const OCR2_AUTO_REGION_BOTTOM_RATIO: f64 = 0.30;
const OCR2_AUTO_REGION_TOP_RATIO: f64 = 0.84;
const OCR2_AUTO_REGION_SLICE_COUNT: u32 = 3;
const OCR2_AUTO_REGION_TARGET_PIXELS: f64 = 2_250_000.0;
const OCR2_AUTO_REGION_MAX_SLICES: u32 = 3;
const OCR2_AUTO_REGION_MAX_SLICE_CAP: u32 = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HybridPdfOcr2RegionPlanner {
    Disabled,
    ProfileRiskWindow,
    ProfileRiskWindowSlices,
    ProfileRiskWindowAdaptive,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct HybridPdfOcr2RegionPatchSizing {
    pub(crate) target_pixels: f64,
    pub(crate) max_slices: u32,
}

pub(crate) async fn render_hybrid_page_ocr_shards(
    source: &Path,
    output: &Path,
) -> Result<PdfPageRenderShardReport, String> {
    let selection = hybrid_page_ocr_render_selection();
    let primary_selection = match selection {
        PdfPageRenderSelection::RegionShards => PdfPageRenderSelection::ShardFallbackPages,
        other => other,
    };
    let requires_rendered_page_images =
        hybrid_page_ocr_profile_planner().requires_rendered_page_images();
    let render_profile = hybrid_page_ocr_render_profile(requires_rendered_page_images);
    let source_for_render = source.to_path_buf();
    let output_for_render = output.to_path_buf();
    tokio::task::spawn_blocking(move || {
        if requires_rendered_page_images {
            #[cfg(feature = "document-extract-pdf-render")]
            return render_pdf_page_shards_with_selection(
                source_for_render.as_path(),
                output_for_render.as_path(),
                &render_profile,
                primary_selection,
            );
            #[cfg(not(feature = "document-extract-pdf-render"))]
            return Err(format!(
                "hosted VLM direct planner for `{}` requires the `document-extract-pdf-render` feature",
                source_for_render.display()
            ));
        }
        prepare_pdf_source_page_range_ocr_shards_with_selection(
            source_for_render.as_path(),
            output_for_render.as_path(),
            &render_profile,
            primary_selection,
        )
    })
    .await
    .map_err(|error| format!("join hybrid PDF OCR render task: {error}"))?
}

fn hybrid_page_ocr_render_profile(ocr2_rendered_page_images: bool) -> PdfPageRenderProfile {
    hybrid_page_ocr_render_profile_with_lookup(ocr2_rendered_page_images, &|key| {
        std::env::var(key).ok()
    })
}

pub(crate) fn hybrid_page_ocr_render_profile_with_lookup(
    ocr2_rendered_page_images: bool,
    lookup: &dyn Fn(&str) -> Option<String>,
) -> PdfPageRenderProfile {
    let mut profile = PdfPageRenderProfile::ocr_default();
    if !ocr2_rendered_page_images {
        return profile;
    }
    let Some(dpi) = lookup(DOCUMENT_EXTRACT_PDF_HOSTED_VLM_RENDER_DPI_ENV)
        .and_then(|value| value.trim().parse::<u32>().ok())
        .filter(|value| *value >= profile.dpi)
    else {
        return profile;
    };
    profile.dpi = dpi;
    profile
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
    let regions = matching_regions.ok_or_else(|| {
        format!(
            "no hybrid PDF region fixture matched source `{}`",
            source.display()
        )
    })?;
    let context_ratio = hybrid_page_ocr_region_context_ratio_with_lookup(lookup);
    Ok(apply_region_context_padding(regions, context_ratio))
}

#[cfg(test)]
pub(crate) fn automatic_ocr2_recovery_region_requests_with_lookup(
    inputs: &[PdfOcrShardInput],
    lookup: &dyn Fn(&str) -> Option<String>,
) -> Vec<PdfPageRegionRenderRequest> {
    automatic_ocr2_recovery_region_requests_for_profiles_with_lookup(inputs, &[], lookup)
}

pub(crate) fn automatic_ocr2_recovery_region_requests_for_source_with_lookup(
    source: &Path,
    inputs: &[PdfOcrShardInput],
    lookup: &dyn Fn(&str) -> Option<String>,
) -> Vec<PdfPageRegionRenderRequest> {
    let profiles = match source_pdf_page_profiles_cached(source) {
        Ok(profiles) => profiles,
        Err(error) => {
            log::debug!("hybrid PDF hosted VLM/OCR region planner skipped source profile: {error}");
            Vec::new()
        }
    };
    automatic_ocr2_recovery_region_requests_for_profiles_with_lookup(
        inputs,
        profiles.as_slice(),
        lookup,
    )
}

pub(crate) fn automatic_ocr2_recovery_region_requests_for_profiles_with_lookup(
    inputs: &[PdfOcrShardInput],
    profiles: &[PdfSourcePageProfile],
    lookup: &dyn Fn(&str) -> Option<String>,
) -> Vec<PdfPageRegionRenderRequest> {
    let planner = hybrid_page_ocr2_region_planner_with_lookup(lookup);
    if planner == HybridPdfOcr2RegionPlanner::Disabled {
        return Vec::new();
    }
    let profiles_by_page = profiles
        .iter()
        .map(|profile| (profile.page_index, profile))
        .collect::<BTreeMap<_, _>>();
    let patch_sizing = hybrid_page_ocr2_region_patch_sizing_with_lookup(lookup);
    let regions = inputs
        .iter()
        .flat_map(|input| match planner {
            HybridPdfOcr2RegionPlanner::ProfileRiskWindow => {
                ocr2_recovery_content_band_region(input)
                    .into_iter()
                    .collect()
            }
            HybridPdfOcr2RegionPlanner::ProfileRiskWindowSlices => {
                ocr2_recovery_content_slices(input)
            }
            HybridPdfOcr2RegionPlanner::ProfileRiskWindowAdaptive => {
                ocr2_recovery_content_adaptive_slices(
                    input,
                    profiles_by_page.get(&input.page_index).copied(),
                    patch_sizing,
                )
            }
            HybridPdfOcr2RegionPlanner::Disabled => Vec::new(),
        })
        .collect::<Vec<_>>();
    let context_ratio = hybrid_page_ocr_region_context_ratio_with_lookup(lookup);
    apply_region_context_padding(regions, context_ratio)
}

pub(crate) fn hybrid_page_ocr2_region_planner_with_lookup(
    lookup: &dyn Fn(&str) -> Option<String>,
) -> HybridPdfOcr2RegionPlanner {
    match lookup(DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_PLANNER_ENV)
        .unwrap_or_default()
        .trim()
        .replace('_', "-")
        .to_ascii_lowercase()
        .as_str()
    {
        OCR2_REGION_PLANNER_PROFILE_RISK_WINDOW => HybridPdfOcr2RegionPlanner::ProfileRiskWindow,
        OCR2_REGION_PLANNER_PROFILE_RISK_WINDOW_SLICES => {
            HybridPdfOcr2RegionPlanner::ProfileRiskWindowSlices
        }
        OCR2_REGION_PLANNER_PROFILE_RISK_WINDOW_ADAPTIVE => {
            HybridPdfOcr2RegionPlanner::ProfileRiskWindowAdaptive
        }
        _ => HybridPdfOcr2RegionPlanner::Disabled,
    }
}

pub(crate) fn hybrid_page_ocr2_region_patch_sizing_with_lookup(
    lookup: &dyn Fn(&str) -> Option<String>,
) -> HybridPdfOcr2RegionPatchSizing {
    let target_pixels = lookup(DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_TARGET_PIXELS_ENV)
        .and_then(|value| value.trim().parse::<f64>().ok())
        .filter(|value| value.is_finite() && *value > 0.0)
        .unwrap_or(OCR2_AUTO_REGION_TARGET_PIXELS);
    let max_slices = lookup(DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_MAX_SLICES_ENV)
        .and_then(|value| value.trim().parse::<u32>().ok())
        .filter(|value| *value > 0)
        .map_or(OCR2_AUTO_REGION_MAX_SLICES, |value| {
            value.min(OCR2_AUTO_REGION_MAX_SLICE_CAP)
        });
    HybridPdfOcr2RegionPatchSizing {
        target_pixels,
        max_slices,
    }
}

fn ocr2_recovery_content_band_region(
    input: &PdfOcrShardInput,
) -> Option<PdfPageRegionRenderRequest> {
    if input.shard_type != "page" || !is_hosted_vlm_direct_profile(input.ocr_profile.as_str()) {
        return None;
    }
    if is_hosted_vlm_topup_page(input) {
        return None;
    }
    let width = input.crop_right - input.crop_left;
    let height = input.crop_top - input.crop_bottom;
    if !width.is_finite() || !height.is_finite() || width <= 0.0 || height <= 0.0 {
        return None;
    }
    let region_box = PdfPageBox::new(
        input.crop_left + width * OCR2_AUTO_REGION_LEFT_RATIO,
        input.crop_bottom + height * OCR2_AUTO_REGION_BOTTOM_RATIO,
        input.crop_left + width * OCR2_AUTO_REGION_RIGHT_RATIO,
        input.crop_bottom + height * OCR2_AUTO_REGION_TOP_RATIO,
    );
    Some(PdfPageRegionRenderRequest::new(
        input.page_index,
        1,
        region_box,
        Some(format!("{:06}.000050", input.page_index)),
    ))
}

fn ocr2_recovery_content_slices(input: &PdfOcrShardInput) -> Vec<PdfPageRegionRenderRequest> {
    let Some(band) = ocr2_recovery_content_band_region(input) else {
        return Vec::new();
    };
    ocr2_recovery_content_band_slices(&band, OCR2_AUTO_REGION_SLICE_COUNT)
}

fn ocr2_recovery_content_adaptive_slices(
    input: &PdfOcrShardInput,
    profile: Option<&PdfSourcePageProfile>,
    patch_sizing: HybridPdfOcr2RegionPatchSizing,
) -> Vec<PdfPageRegionRenderRequest> {
    let Some(band) = ocr2_recovery_content_band_region(input) else {
        return Vec::new();
    };
    let content_pixels = estimated_region_pixels(input, band.region_box);
    let slice_count = adaptive_region_slice_count(content_pixels, profile, patch_sizing);
    ocr2_recovery_content_band_slices(&band, slice_count)
}

fn adaptive_region_slice_count(
    content_pixels: f64,
    profile: Option<&PdfSourcePageProfile>,
    patch_sizing: HybridPdfOcr2RegionPatchSizing,
) -> u32 {
    let pixel_slices = pixel_area_slice_count(content_pixels, patch_sizing);
    let Some(profile) = profile else {
        return pixel_slices;
    };
    if is_exact_structural_risk(profile) {
        return pixel_slices.max(3).min(patch_sizing.max_slices);
    }
    if pixel_slices == 1 && is_low_complexity_risk_neighbor(profile) {
        return 1;
    }
    pixel_slices
}

fn pixel_area_slice_count(
    content_pixels: f64,
    patch_sizing: HybridPdfOcr2RegionPatchSizing,
) -> u32 {
    if !content_pixels.is_finite() || content_pixels <= 0.0 {
        return 1;
    }
    (content_pixels / patch_sizing.target_pixels)
        .ceil()
        .max(1.0)
        .min(f64::from(patch_sizing.max_slices)) as u32
}

fn is_exact_structural_risk(profile: &PdfSourcePageProfile) -> bool {
    let compact_table_grid = (1..=8).contains(&profile.rectangle_ops)
        && profile.operation_count >= 640
        && profile.text_show_ops >= 120;
    let dense_table_path_band = (64..=120).contains(&profile.path_ops)
        && profile.operation_count >= 640
        && profile.text_show_ops >= 150;
    compact_table_grid || dense_table_path_band
}

fn is_low_complexity_risk_neighbor(profile: &PdfSourcePageProfile) -> bool {
    profile.operation_count < 320
        && profile.text_show_ops < 100
        && profile.path_ops < 48
        && profile.rectangle_ops == 0
        && profile.draw_object_ops <= 1
}

fn ocr2_recovery_content_band_slices(
    band: &PdfPageRegionRenderRequest,
    slice_count: u32,
) -> Vec<PdfPageRegionRenderRequest> {
    let height = band.region_box.top - band.region_box.bottom;
    if !height.is_finite() || height <= 0.0 || slice_count == 0 {
        return Vec::new();
    }
    let slice_height = height / f64::from(slice_count);
    (0..slice_count)
        .map(|slice| {
            let top = band.region_box.top - f64::from(slice) * slice_height;
            let bottom = (top - slice_height).max(band.region_box.bottom);
            let region_index = slice.saturating_add(1);
            let order_offset = slice_reading_order_offset(slice, slice_count);
            PdfPageRegionRenderRequest::new(
                band.page_index,
                region_index,
                PdfPageBox::new(band.region_box.left, bottom, band.region_box.right, top),
                Some(format!("{:06}.{:06}", band.page_index, order_offset)),
            )
        })
        .collect()
}

fn slice_reading_order_offset(slice: u32, slice_count: u32) -> u32 {
    match slice_count {
        2 => {
            if slice == 0 {
                40
            } else {
                60
            }
        }
        3 => 30_u32.saturating_add(slice.saturating_mul(20)),
        _ => 50,
    }
}

fn estimated_region_pixels(input: &PdfOcrShardInput, region_box: PdfPageBox) -> f64 {
    let scale_x = if input.point_to_pixel_scale_x.is_finite() && input.point_to_pixel_scale_x > 0.0
    {
        input.point_to_pixel_scale_x
    } else {
        page_scale_from_raster(input.raster_width_px, input.crop_right - input.crop_left)
    };
    let scale_y = if input.point_to_pixel_scale_y.is_finite() && input.point_to_pixel_scale_y > 0.0
    {
        input.point_to_pixel_scale_y
    } else {
        page_scale_from_raster(input.raster_height_px, input.crop_top - input.crop_bottom)
    };
    let width = region_box.width_points() * scale_x;
    let height = region_box.height_points() * scale_y;
    if !width.is_finite() || !height.is_finite() || width <= 0.0 || height <= 0.0 {
        0.0
    } else {
        width * height
    }
}

fn page_scale_from_raster(raster_px: u32, page_points: f64) -> f64 {
    if page_points.is_finite() && page_points > 0.0 {
        f64::from(raster_px) / page_points
    } else {
        0.0
    }
}

fn paths_match(left: &Path, right: &Path) -> bool {
    left == right
        || match (left.canonicalize(), right.canonicalize()) {
            (Ok(left), Ok(right)) => left == right,
            _ => false,
        }
}

pub(crate) fn hybrid_page_ocr_region_context_ratio_with_lookup(
    lookup: &dyn Fn(&str) -> Option<String>,
) -> f64 {
    lookup(DOCUMENT_EXTRACT_PDF_REGION_CONTEXT_RATIO_ENV)
        .and_then(|value| value.trim().parse::<f64>().ok())
        .filter(|value| value.is_finite() && *value >= 0.0)
        .map_or(DEFAULT_OCR2_REGION_CONTEXT_RATIO, |value| value.min(1.0))
}

fn apply_region_context_padding(
    regions: Vec<PdfPageRegionRenderRequest>,
    context_ratio: f64,
) -> Vec<PdfPageRegionRenderRequest> {
    if context_ratio <= 0.0 {
        return regions;
    }
    regions
        .into_iter()
        .map(|mut request| {
            request.region_box = padded_region_box(request.region_box, context_ratio);
            request
        })
        .collect()
}

fn padded_region_box(box_points: PdfPageBox, context_ratio: f64) -> PdfPageBox {
    let pad_x = box_points.width_points() * context_ratio;
    let pad_y = box_points.height_points() * context_ratio;
    PdfPageBox::new(
        box_points.left - pad_x,
        box_points.bottom - pad_y,
        box_points.right + pad_x,
        box_points.top + pad_y,
    )
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
            "render status `{}` is not eligible for hybrid OCR{}",
            report.status,
            render_report_error_suffix(report),
        ));
    }
    if report.routing_decision != PdfRenderRoutingDecision::HybridPageOcrCandidate.as_str() {
        return Err(format!(
            "routing decision `{}` is not eligible for hybrid OCR{}",
            report.routing_decision,
            render_report_error_suffix(report),
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

fn render_report_error_suffix(report: &PdfPageRenderShardReport) -> String {
    report
        .error_message
        .as_deref()
        .filter(|message| !message.trim().is_empty())
        .map(|message| format!(": {message}"))
        .unwrap_or_default()
}
