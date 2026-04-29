use std::fmt::Write as _;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use arrow::array::{ArrayRef, Float64Array, Int32Array, StringArray};
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use arrow::ipc::writer::FileWriter;
use arrow::record_batch::RecordBatch;
use image::DynamicImage;
use num_traits::ToPrimitive;
use pdfium_render::prelude::{
    PdfBitmapFormat, PdfDocument, PdfPage, PdfPageRenderRotation, PdfRect, PdfRenderConfig, Pdfium,
    PdfiumError,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::audit::{
    PdfInspectorPdfType, PdfInspectorRoutingDecision, PdfInspectorRoutingSignals,
    analyze_pdf_routing_signals, high_recall_ocr_page_numbers, routing_assessment,
};
use super::ocr::{PdfOcrWorkerProfile, build_ocr_shard_input_batch, build_ocr_shard_inputs};

pub const PDFIUM_LIBRARY_PATH_ENV: &str = "WENDAO_PDFIUM_LIBRARY_PATH";
const PDF_RENDER_SHARD_PROFILE: &str = "pdfium-render-page-shards-v1";
const OCR_SHARD_MANIFEST_ARROW_NAME: &str = "_ocr_shards.arrow";
const OCR_SHARD_INPUT_ARROW_NAME: &str = "_ocr_input.arrow";
const OCR_PENDING_RESOURCE_ARROW_NAME: &str = "_ocr_pending.arrow";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PdfRenderRoutingDecision {
    FastRustCandidate,
    HybridPageOcrCandidate,
    FullDoclingFallback,
    PreflightFailed,
    UnsupportedNonPdf,
}

impl PdfRenderRoutingDecision {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::FastRustCandidate => "fast_rust_candidate",
            Self::HybridPageOcrCandidate => "hybrid_page_ocr_candidate",
            Self::FullDoclingFallback => "full_docling_fallback",
            Self::PreflightFailed => "preflight_failed",
            Self::UnsupportedNonPdf => "unsupported_non_pdf",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PdfRenderStatus {
    Rendered,
    Fallback,
    Skipped,
    Unsupported,
}

impl PdfRenderStatus {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Rendered => "rendered",
            Self::Fallback => "fallback",
            Self::Skipped => "skipped",
            Self::Unsupported => "unsupported",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PdfPageRenderSelection {
    AllPages,
    ShardFallbackPages,
    RegionShards,
}

impl PdfPageRenderSelection {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::AllPages => "all_pages",
            Self::ShardFallbackPages => "shard_fallback_pages",
            Self::RegionShards => "region_shards",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfPageRenderProfile {
    pub profile_id: String,
    pub dpi: u32,
    pub image_extension: String,
    pub image_mime_type: String,
    pub render_annotations: bool,
    pub render_form_data: bool,
}

impl PdfPageRenderProfile {
    #[must_use]
    pub fn ocr_default() -> Self {
        Self {
            profile_id: PDF_RENDER_SHARD_PROFILE.to_string(),
            dpi: 300,
            image_extension: "png".to_string(),
            image_mime_type: "image/png".to_string(),
            render_annotations: true,
            render_form_data: true,
        }
    }
}

impl Default for PdfPageRenderProfile {
    fn default() -> Self {
        Self::ocr_default()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfPageBox {
    pub left: f64,
    pub bottom: f64,
    pub right: f64,
    pub top: f64,
}

impl PdfPageBox {
    #[must_use]
    pub fn new(left: f64, bottom: f64, right: f64, top: f64) -> Self {
        let (left, right) = if left <= right {
            (left, right)
        } else {
            (right, left)
        };
        let (bottom, top) = if bottom <= top {
            (bottom, top)
        } else {
            (top, bottom)
        };
        Self {
            left,
            bottom,
            right,
            top,
        }
    }

    #[must_use]
    pub fn width_points(&self) -> f64 {
        self.right - self.left
    }

    #[must_use]
    pub fn height_points(&self) -> f64 {
        self.top - self.bottom
    }

    #[must_use]
    pub fn intersection(&self, other: Self) -> Option<Self> {
        let left = self.left.max(other.left);
        let bottom = self.bottom.max(other.bottom);
        let right = self.right.min(other.right);
        let top = self.top.min(other.top);
        (left < right && bottom < top).then(|| Self::new(left, bottom, right, top))
    }

    fn from_pdfium_rect(rect: PdfRect) -> Self {
        Self::new(
            f64::from(rect.left().value),
            f64::from(rect.bottom().value),
            f64::from(rect.right().value),
            f64::from(rect.top().value),
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PdfOcrShardType {
    Page,
    Region,
}

impl PdfOcrShardType {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Page => "page",
            Self::Region => "region",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfPagePixelBox {
    pub left: u32,
    pub top: u32,
    pub right: u32,
    pub bottom: u32,
}

impl PdfPagePixelBox {
    #[must_use]
    pub fn new(left: u32, top: u32, right: u32, bottom: u32) -> Self {
        Self {
            left,
            top,
            right,
            bottom,
        }
    }

    #[must_use]
    pub fn width_px(&self) -> u32 {
        self.right.saturating_sub(self.left)
    }

    #[must_use]
    pub fn height_px(&self) -> u32 {
        self.bottom.saturating_sub(self.top)
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfPageRegion {
    pub region_index: u32,
    pub region_box: PdfPageBox,
    pub parent_shard_element_id: String,
    pub reading_order_key: String,
}

impl PdfPageRegion {
    #[must_use]
    pub fn new(
        region_index: u32,
        region_box: PdfPageBox,
        parent_shard_element_id: impl Into<String>,
        reading_order_key: impl Into<String>,
    ) -> Self {
        Self {
            region_index,
            region_box,
            parent_shard_element_id: parent_shard_element_id.into(),
            reading_order_key: reading_order_key.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfPageShardGeometry {
    pub media_box: PdfPageBox,
    pub crop_box: PdfPageBox,
    pub rotation_degrees: u16,
    pub render_dpi: u32,
    pub raster_width_px: u32,
    pub raster_height_px: u32,
    pub point_to_pixel_scale_x: f64,
    pub point_to_pixel_scale_y: f64,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfPageShardManifest {
    pub source_path: String,
    pub source_content_hash: String,
    pub page_index: u32,
    pub shard_type: PdfOcrShardType,
    pub region_index: u32,
    pub parent_shard_element_id: String,
    pub reading_order_key: String,
    pub render_profile: String,
    pub image_path: String,
    pub image_mime_type: String,
    pub raster_sha256: String,
    pub geometry: PdfPageShardGeometry,
    pub source_page_pixel_box: PdfPagePixelBox,
    pub element_id: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfPageRenderShardReport {
    pub source_path: String,
    pub output_dir: String,
    pub page_count: u32,
    pub shard_count: u32,
    pub manifest_arrow_path: Option<String>,
    pub ocr_input_arrow_path: Option<String>,
    pub pending_resource_arrow_path: Option<String>,
    pub render_profile: String,
    pub render_selection: String,
    pub status: String,
    pub routing_decision: String,
    pub elapsed_ms: f64,
    pub error_message: Option<String>,
}

#[must_use]
pub fn render_dimensions_for_box(
    page_box: PdfPageBox,
    rotation_degrees: u16,
    profile: &PdfPageRenderProfile,
) -> (u32, u32) {
    let width_px = points_to_pixels(page_box.width_points(), profile.dpi);
    let height_px = points_to_pixels(page_box.height_points(), profile.dpi);
    if rotation_degrees % 180 == 90 {
        (height_px, width_px)
    } else {
        (width_px, height_px)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PdfPageShardManifestInput<'a> {
    pub source_path: &'a Path,
    pub source_content_hash: &'a str,
    pub page_index: u32,
    pub profile: &'a PdfPageRenderProfile,
    pub media_box: PdfPageBox,
    pub crop_box: PdfPageBox,
    pub rotation_degrees: u16,
    pub raster: RenderedRasterIdentity,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PdfPageRegionShardManifestInput<'a> {
    pub source_path: &'a Path,
    pub source_content_hash: &'a str,
    pub page_index: u32,
    pub profile: &'a PdfPageRenderProfile,
    pub media_box: PdfPageBox,
    pub page_crop_box: PdfPageBox,
    pub region: PdfPageRegion,
    pub rotation_degrees: u16,
    pub page_raster_width_px: u32,
    pub page_raster_height_px: u32,
    pub raster: RenderedRasterIdentity,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfPageRegionRenderRequest {
    pub page_index: u32,
    pub region_index: u32,
    pub region_box: PdfPageBox,
    pub reading_order_key: Option<String>,
}

impl PdfPageRegionRenderRequest {
    #[must_use]
    pub fn new(
        page_index: u32,
        region_index: u32,
        region_box: PdfPageBox,
        reading_order_key: Option<String>,
    ) -> Self {
        Self {
            page_index,
            region_index,
            region_box,
            reading_order_key,
        }
    }

    fn effective_reading_order_key(&self) -> String {
        self.reading_order_key
            .clone()
            .unwrap_or_else(|| region_reading_order_key(self.page_index, self.region_index))
    }
}

#[must_use]
pub fn build_shard_manifest(input: PdfPageShardManifestInput<'_>) -> PdfPageShardManifest {
    let element_id = shard_element_id(
        input.source_content_hash,
        input.page_index,
        &input.profile.profile_id,
    );
    let scale_x = f64::from(input.raster.width_px) / input.crop_box.width_points().max(1.0);
    let scale_y = f64::from(input.raster.height_px) / input.crop_box.height_points().max(1.0);
    PdfPageShardManifest {
        source_path: input.source_path.to_string_lossy().to_string(),
        source_content_hash: input.source_content_hash.to_string(),
        page_index: input.page_index,
        shard_type: PdfOcrShardType::Page,
        region_index: 0,
        parent_shard_element_id: String::new(),
        reading_order_key: page_reading_order_key(input.page_index),
        render_profile: input.profile.profile_id.clone(),
        image_path: input.raster.path.to_string_lossy().to_string(),
        image_mime_type: input.profile.image_mime_type.clone(),
        raster_sha256: input.raster.sha256,
        geometry: PdfPageShardGeometry {
            media_box: input.media_box,
            crop_box: input.crop_box,
            rotation_degrees: input.rotation_degrees,
            render_dpi: input.profile.dpi,
            raster_width_px: input.raster.width_px,
            raster_height_px: input.raster.height_px,
            point_to_pixel_scale_x: scale_x,
            point_to_pixel_scale_y: scale_y,
        },
        source_page_pixel_box: PdfPagePixelBox::new(
            0,
            0,
            input.raster.width_px,
            input.raster.height_px,
        ),
        element_id,
    }
}

/// # Errors
///
/// Returns an error if the requested region does not intersect the source page
/// crop box or cannot be represented in source-page pixel coordinates.
pub fn build_region_shard_manifest(
    input: PdfPageRegionShardManifestInput<'_>,
) -> Result<PdfPageShardManifest, String> {
    let region_box = input
        .region
        .region_box
        .intersection(input.page_crop_box)
        .ok_or_else(|| {
            format!(
                "region {} does not intersect page {} crop box",
                input.region.region_index, input.page_index
            )
        })?;
    let source_page_pixel_box = region_pixel_box_for_crop(
        input.page_crop_box,
        region_box,
        input.page_raster_width_px,
        input.page_raster_height_px,
    )?;
    let element_id = region_shard_element_id(
        input.source_content_hash,
        input.page_index,
        &input.profile.profile_id,
        input.region.region_index,
        region_box,
    );
    let scale_x = f64::from(input.raster.width_px) / region_box.width_points().max(1.0);
    let scale_y = f64::from(input.raster.height_px) / region_box.height_points().max(1.0);
    Ok(PdfPageShardManifest {
        source_path: input.source_path.to_string_lossy().to_string(),
        source_content_hash: input.source_content_hash.to_string(),
        page_index: input.page_index,
        shard_type: PdfOcrShardType::Region,
        region_index: input.region.region_index,
        parent_shard_element_id: input.region.parent_shard_element_id,
        reading_order_key: input.region.reading_order_key,
        render_profile: input.profile.profile_id.clone(),
        image_path: input.raster.path.to_string_lossy().to_string(),
        image_mime_type: input.profile.image_mime_type.clone(),
        raster_sha256: input.raster.sha256,
        geometry: PdfPageShardGeometry {
            media_box: input.media_box,
            crop_box: region_box,
            rotation_degrees: input.rotation_degrees,
            render_dpi: input.profile.dpi,
            raster_width_px: input.raster.width_px,
            raster_height_px: input.raster.height_px,
            point_to_pixel_scale_x: scale_x,
            point_to_pixel_scale_y: scale_y,
        },
        source_page_pixel_box,
        element_id,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderedRasterIdentity {
    pub path: PathBuf,
    pub sha256: String,
    pub width_px: u32,
    pub height_px: u32,
}

/// # Errors
///
/// Returns an error if Arrow cannot build a typed shard manifest batch.
pub fn build_shard_manifest_batch(
    manifests: &[PdfPageShardManifest],
) -> Result<RecordBatch, String> {
    RecordBatch::try_new(
        shard_manifest_schema(),
        vec![
            string_manifest_column(manifests, |manifest| manifest.source_path.clone()),
            string_manifest_column(manifests, |manifest| manifest.source_content_hash.clone()),
            int_manifest_column(manifests, |manifest| manifest.page_index),
            string_manifest_column(manifests, |manifest| manifest.render_profile.clone()),
            string_manifest_column(manifests, |manifest| manifest.image_path.clone()),
            string_manifest_column(manifests, |manifest| manifest.image_mime_type.clone()),
            string_manifest_column(manifests, |manifest| manifest.raster_sha256.clone()),
            int_manifest_column(manifests, |manifest| manifest.geometry.raster_width_px),
            int_manifest_column(manifests, |manifest| manifest.geometry.raster_height_px),
            int_manifest_column(manifests, |manifest| manifest.geometry.render_dpi),
            Arc::new(Int32Array::from(
                manifests
                    .iter()
                    .map(|manifest| i32::from(manifest.geometry.rotation_degrees))
                    .collect::<Vec<_>>(),
            )),
            float_manifest_column(manifests, |manifest| manifest.geometry.media_box.left),
            float_manifest_column(manifests, |manifest| manifest.geometry.media_box.bottom),
            float_manifest_column(manifests, |manifest| manifest.geometry.media_box.right),
            float_manifest_column(manifests, |manifest| manifest.geometry.media_box.top),
            float_manifest_column(manifests, |manifest| manifest.geometry.crop_box.left),
            float_manifest_column(manifests, |manifest| manifest.geometry.crop_box.bottom),
            float_manifest_column(manifests, |manifest| manifest.geometry.crop_box.right),
            float_manifest_column(manifests, |manifest| manifest.geometry.crop_box.top),
            float_manifest_column(manifests, |manifest| {
                manifest.geometry.point_to_pixel_scale_x
            }),
            float_manifest_column(manifests, |manifest| {
                manifest.geometry.point_to_pixel_scale_y
            }),
            string_manifest_column(manifests, |manifest| manifest.element_id.clone()),
            string_manifest_column(manifests, |manifest| {
                manifest.shard_type.as_str().to_string()
            }),
            int_manifest_column(manifests, |manifest| manifest.region_index),
            string_manifest_column(manifests, |manifest| {
                manifest.parent_shard_element_id.clone()
            }),
            string_manifest_column(manifests, |manifest| manifest.reading_order_key.clone()),
            int_manifest_column(manifests, |manifest| manifest.source_page_pixel_box.left),
            int_manifest_column(manifests, |manifest| manifest.source_page_pixel_box.top),
            int_manifest_column(manifests, |manifest| manifest.source_page_pixel_box.right),
            int_manifest_column(manifests, |manifest| manifest.source_page_pixel_box.bottom),
        ],
    )
    .map_err(|error| format!("build OCR shard manifest Arrow batch: {error}"))
}

/// # Errors
///
/// Returns an error if Arrow cannot build the stable document-resource batch.
pub fn build_ocr_pending_resource_batch(
    manifests: &[PdfPageShardManifest],
) -> Result<RecordBatch, String> {
    RecordBatch::try_new(
        document_resource_schema(),
        vec![
            Arc::new(StringArray::from(
                manifests
                    .iter()
                    .map(|manifest| manifest.source_path.as_str())
                    .collect::<Vec<_>>(),
            )) as ArrayRef,
            Arc::new(StringArray::from(vec!["ocr_pending"; manifests.len()])),
            Arc::new(StringArray::from(
                manifests
                    .iter()
                    .map(|manifest| manifest.image_path.as_str())
                    .collect::<Vec<_>>(),
            )),
            Arc::new(Int32Array::from(
                manifests
                    .iter()
                    .map(|manifest| i32::try_from(manifest.page_index).unwrap_or(i32::MAX))
                    .collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                manifests
                    .iter()
                    .map(pending_caption)
                    .collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                manifests
                    .iter()
                    .map(|manifest| {
                        format!(
                            "manifest={},raster_sha256={},profile={},shard_type={},reading_order_key={}",
                            OCR_SHARD_MANIFEST_ARROW_NAME,
                            manifest.raster_sha256,
                            manifest.render_profile,
                            manifest.shard_type.as_str(),
                            manifest.reading_order_key
                        )
                    })
                    .collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                manifests
                    .iter()
                    .map(|manifest| manifest.image_mime_type.as_str())
                    .collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(vec!["pending"; manifests.len()])),
            Arc::new(StringArray::from(
                manifests
                    .iter()
                    .map(|manifest| manifest.element_id.as_str())
                    .collect::<Vec<_>>(),
            )),
        ],
    )
    .map_err(|error| format!("build OCR pending resource Arrow batch: {error}"))
}

/// # Errors
///
/// Returns an error if the path cannot be read or Arrow report files cannot be
/// written. Missing `PDFium` libraries are represented as fallback reports rather
/// than errors.
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
/// Returns an error if the PDF cannot be read, the requested regions cannot be
/// rendered, or Arrow artifact files cannot be written. Missing `PDFium`
/// libraries are represented as fallback reports rather than errors.
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

/// # Errors
///
/// Returns an error if reports cannot be written.
pub fn write_page_render_shard_reports(
    report_dir: &Path,
    records: &[PdfPageRenderShardReport],
) -> Result<(), String> {
    fs::create_dir_all(report_dir)
        .map_err(|error| format!("create report dir `{}`: {error}", report_dir.display()))?;
    let json_path = report_dir.join("pdf_page_render_shard_manifest.json");
    let report = serde_json::json!({
        "schema": "xiuxian_wendao.pdf_page_render_shard_manifest.v1",
        "profile": PDF_RENDER_SHARD_PROFILE,
        "totalInputs": records.len(),
        "totalRenderedShards": records.iter().map(|record| record.shard_count).sum::<u32>(),
        "renderedInputs": records.iter().filter(|record| record.status == "rendered").count(),
        "fallbackInputs": records.iter().filter(|record| record.status == "fallback").count(),
        "records": records,
    });
    fs::write(
        json_path.as_path(),
        serde_json::to_vec_pretty(&report).map_err(|error| error.to_string())?,
    )
    .map_err(|error| format!("write report `{}`: {error}", json_path.display()))?;

    let markdown_path = report_dir.join("pdf_page_render_shard_manifest.md");
    fs::write(markdown_path.as_path(), render_markdown_report(records))
        .map_err(|error| format!("write report `{}`: {error}", markdown_path.display()))?;
    Ok(())
}

fn points_to_pixels(points: f64, dpi: u32) -> u32 {
    ((points / 72.0) * f64::from(dpi))
        .round()
        .max(1.0)
        .to_u32()
        .unwrap_or(u32::MAX)
}

fn rotation_to_degrees(rotation: PdfPageRenderRotation) -> u16 {
    match rotation {
        PdfPageRenderRotation::None => 0,
        PdfPageRenderRotation::Degrees90 => 90,
        PdfPageRenderRotation::Degrees180 => 180,
        PdfPageRenderRotation::Degrees270 => 270,
    }
}

fn shard_element_id(content_hash: &str, page_index: u32, profile_id: &str) -> String {
    sha256_hex(format!("{content_hash}:{page_index}:{profile_id}").as_bytes())
}

fn region_shard_element_id(
    content_hash: &str,
    page_index: u32,
    profile_id: &str,
    region_index: u32,
    region_box: PdfPageBox,
) -> String {
    sha256_hex(
        format!(
            "{content_hash}:{page_index}:{profile_id}:region:{region_index}:{:.6}:{:.6}:{:.6}:{:.6}",
            region_box.left, region_box.bottom, region_box.right, region_box.top
        )
        .as_bytes(),
    )
}

fn page_reading_order_key(page_index: u32) -> String {
    format!("{page_index:06}.000000")
}

fn region_reading_order_key(page_index: u32, region_index: u32) -> String {
    format!("{page_index:06}.{region_index:06}")
}

fn pending_caption(manifest: &PdfPageShardManifest) -> String {
    match manifest.shard_type {
        PdfOcrShardType::Page => format!("OCR pending PDF page {}", manifest.page_index + 1),
        PdfOcrShardType::Region => format!(
            "OCR pending PDF page {} region {}",
            manifest.page_index + 1,
            manifest.region_index
        ),
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn checked_len_u32(len: usize) -> u32 {
    u32::try_from(len).unwrap_or(u32::MAX)
}

fn checked_pixels_i32(value: u32) -> Result<i32, String> {
    i32::try_from(value).map_err(|_| format!("render target pixel dimension is too large: {value}"))
}

/// # Errors
///
/// Returns an error if the region is outside the page crop box or the source
/// raster dimensions cannot represent a non-empty pixel crop.
pub fn region_pixel_box_for_crop(
    page_crop_box: PdfPageBox,
    region_box: PdfPageBox,
    raster_width_px: u32,
    raster_height_px: u32,
) -> Result<PdfPagePixelBox, String> {
    if raster_width_px == 0 || raster_height_px == 0 {
        return Err("source page raster dimensions must be non-zero".to_string());
    }
    let clipped = region_box
        .intersection(page_crop_box)
        .ok_or_else(|| "region does not intersect page crop box".to_string())?;
    let scale_x = f64::from(raster_width_px) / page_crop_box.width_points().max(1.0);
    let scale_y = f64::from(raster_height_px) / page_crop_box.height_points().max(1.0);
    let left = floor_pixel(
        (clipped.left - page_crop_box.left) * scale_x,
        raster_width_px,
    );
    let right = ceil_pixel(
        (clipped.right - page_crop_box.left) * scale_x,
        raster_width_px,
    );
    let top = floor_pixel(
        (page_crop_box.top - clipped.top) * scale_y,
        raster_height_px,
    );
    let bottom = ceil_pixel(
        (page_crop_box.top - clipped.bottom) * scale_y,
        raster_height_px,
    );
    if left >= right || top >= bottom {
        return Err("region maps to an empty source page pixel box".to_string());
    }
    Ok(PdfPagePixelBox::new(left, top, right, bottom))
}

fn floor_pixel(value: f64, max: u32) -> u32 {
    value
        .floor()
        .clamp(0.0, f64::from(max))
        .to_u32()
        .unwrap_or(max)
}

fn ceil_pixel(value: f64, max: u32) -> u32 {
    value
        .ceil()
        .clamp(0.0, f64::from(max))
        .to_u32()
        .unwrap_or(max)
}

fn is_pdf_path(path: &Path) -> bool {
    path.extension()
        .and_then(|suffix| suffix.to_str())
        .is_some_and(|suffix| suffix.eq_ignore_ascii_case("pdf"))
}

fn string_manifest_column<F>(manifests: &[PdfPageShardManifest], value: F) -> ArrayRef
where
    F: Fn(&PdfPageShardManifest) -> String,
{
    Arc::new(StringArray::from(
        manifests.iter().map(value).collect::<Vec<_>>(),
    ))
}

fn int_manifest_column<F>(manifests: &[PdfPageShardManifest], value: F) -> ArrayRef
where
    F: Fn(&PdfPageShardManifest) -> u32,
{
    Arc::new(Int32Array::from(
        manifests
            .iter()
            .map(|manifest| i32::try_from(value(manifest)).unwrap_or(i32::MAX))
            .collect::<Vec<_>>(),
    ))
}

fn float_manifest_column<F>(manifests: &[PdfPageShardManifest], value: F) -> ArrayRef
where
    F: Fn(&PdfPageShardManifest) -> f64,
{
    Arc::new(Float64Array::from(
        manifests.iter().map(value).collect::<Vec<_>>(),
    ))
}

fn bind_pdfium() -> Result<Pdfium, String> {
    let bindings = match std::env::var(PDFIUM_LIBRARY_PATH_ENV) {
        Ok(path) if !path.trim().is_empty() => Pdfium::bind_to_library(path.as_str()),
        _ => Pdfium::bind_to_system_library(),
    };
    match bindings {
        Ok(bindings) => Ok(Pdfium::new(bindings)),
        Err(PdfiumError::PdfiumLibraryBindingsAlreadyInitialized) => Ok(Pdfium),
        Err(error) => Err(format!("bind Pdfium library: {error}")),
    }
}

enum RenderPageSelection {
    All,
    Selected(Vec<i32>),
    Skip {
        page_count: u32,
        routing_decision: PdfRenderRoutingDecision,
        reason: String,
    },
}

impl RenderPageSelection {
    fn selected_page_indices(&self) -> Option<&[i32]> {
        match self {
            Self::Selected(page_indices) => Some(page_indices.as_slice()),
            Self::All | Self::Skip { .. } => None,
        }
    }
}

fn resolve_page_selection(
    path: &Path,
    selection: PdfPageRenderSelection,
) -> Result<RenderPageSelection, String> {
    match selection {
        PdfPageRenderSelection::AllPages => Ok(RenderPageSelection::All),
        PdfPageRenderSelection::ShardFallbackPages => resolve_shard_fallback_page_selection(path),
        PdfPageRenderSelection::RegionShards => {
            Err("region_shards selection requires configured region requests".to_string())
        }
    }
}

fn resolve_shard_fallback_page_selection(path: &Path) -> Result<RenderPageSelection, String> {
    let signals = analyze_pdf_routing_signals(path)?;
    let assessment = routing_assessment(&signals);
    match assessment.decision {
        PdfInspectorRoutingDecision::FastRustCandidate => Ok(RenderPageSelection::Skip {
            page_count: signals.page_count,
            routing_decision: PdfRenderRoutingDecision::FastRustCandidate,
            reason: "fast text path candidate; raster OCR render is not needed".to_string(),
        }),
        PdfInspectorRoutingDecision::FullDoclingFallback => Ok(RenderPageSelection::Skip {
            page_count: signals.page_count,
            routing_decision: PdfRenderRoutingDecision::FullDoclingFallback,
            reason: "routing gates require full Docling fallback".to_string(),
        }),
        PdfInspectorRoutingDecision::PreflightFailed => Ok(RenderPageSelection::Skip {
            page_count: signals.page_count,
            routing_decision: PdfRenderRoutingDecision::PreflightFailed,
            reason: "PDF preflight failed".to_string(),
        }),
        PdfInspectorRoutingDecision::UnsupportedNonPdf => Ok(RenderPageSelection::Skip {
            page_count: signals.page_count,
            routing_decision: PdfRenderRoutingDecision::UnsupportedNonPdf,
            reason: "unsupported non-PDF input".to_string(),
        }),
        PdfInspectorRoutingDecision::HybridPageOcrCandidate => {
            let pages_needing_ocr = if signals.pages_needing_ocr.is_empty() {
                high_recall_ocr_page_numbers(path)?
            } else {
                signals.pages_needing_ocr.clone()
            };
            let page_indices = raster_ocr_page_indices(
                signals.page_count,
                pages_needing_ocr.as_slice(),
                should_render_all_when_no_ocr_hints(&signals),
            );
            if page_indices.is_empty() {
                return Ok(RenderPageSelection::Skip {
                    page_count: signals.page_count,
                    routing_decision: PdfRenderRoutingDecision::HybridPageOcrCandidate,
                    reason: "hybrid shard fallback selected; no raster OCR pages are required"
                        .to_string(),
                });
            }
            Ok(RenderPageSelection::Selected(page_indices))
        }
    }
}

fn should_render_all_when_no_ocr_hints(signals: &PdfInspectorRoutingSignals) -> bool {
    signals.is_complex
        || matches!(
            signals.pdf_type,
            PdfInspectorPdfType::Scanned
                | PdfInspectorPdfType::ImageBased
                | PdfInspectorPdfType::Mixed
        )
}

fn raster_ocr_page_indices(
    page_count: u32,
    pages_needing_ocr: &[u32],
    render_all_when_no_hints: bool,
) -> Vec<i32> {
    let mut page_indices = if pages_needing_ocr.is_empty() && render_all_when_no_hints {
        (0..page_count)
            .filter_map(|page_index| i32::try_from(page_index).ok())
            .collect::<Vec<_>>()
    } else {
        pages_needing_ocr
            .iter()
            .filter_map(|page_number| page_number.checked_sub(1))
            .filter(|page_index| *page_index < page_count)
            .filter_map(|page_index| i32::try_from(page_index).ok())
            .collect::<Vec<_>>()
    };
    page_indices.sort_unstable();
    page_indices.dedup();
    page_indices
}

struct RenderShardContext<'a> {
    path: &'a Path,
    output_dir: &'a Path,
    profile: &'a PdfPageRenderProfile,
    selection: PdfPageRenderSelection,
    source_path: String,
    started: Instant,
}

impl<'a> RenderShardContext<'a> {
    fn new(
        path: &'a Path,
        output_dir: &'a Path,
        profile: &'a PdfPageRenderProfile,
        selection: PdfPageRenderSelection,
    ) -> Self {
        Self {
            path,
            output_dir,
            profile,
            selection,
            source_path: path.to_string_lossy().to_string(),
            started: Instant::now(),
        }
    }

    fn report(&self, parts: ReportParts) -> PdfPageRenderShardReport {
        PdfPageRenderShardReport {
            source_path: self.source_path.clone(),
            output_dir: self.output_dir.to_string_lossy().to_string(),
            page_count: parts.page_count,
            shard_count: parts.shard_count,
            manifest_arrow_path: parts
                .manifest_arrow_path
                .map(|path| path.to_string_lossy().to_string()),
            ocr_input_arrow_path: parts
                .ocr_input_arrow_path
                .map(|path| path.to_string_lossy().to_string()),
            pending_resource_arrow_path: parts
                .pending_resource_arrow_path
                .map(|path| path.to_string_lossy().to_string()),
            render_profile: self.profile.profile_id.clone(),
            render_selection: self.selection.as_str().to_string(),
            status: parts.status.as_str().to_string(),
            routing_decision: parts.routing_decision.as_str().to_string(),
            elapsed_ms: self.started.elapsed().as_secs_f64() * 1000.0,
            error_message: parts.error_message,
        }
    }

    fn shard_dir(&self, source_hash: &str) -> PathBuf {
        self.output_dir.join("ocr-shards").join(source_hash)
    }
}

struct ReportParts {
    page_count: u32,
    shard_count: u32,
    manifest_arrow_path: Option<PathBuf>,
    ocr_input_arrow_path: Option<PathBuf>,
    pending_resource_arrow_path: Option<PathBuf>,
    status: PdfRenderStatus,
    routing_decision: PdfRenderRoutingDecision,
    error_message: Option<String>,
}

impl ReportParts {
    fn unsupported(error_message: &str) -> Self {
        Self {
            page_count: 0,
            shard_count: 0,
            manifest_arrow_path: None,
            ocr_input_arrow_path: None,
            pending_resource_arrow_path: None,
            status: PdfRenderStatus::Unsupported,
            routing_decision: PdfRenderRoutingDecision::UnsupportedNonPdf,
            error_message: Some(error_message.to_string()),
        }
    }

    fn fallback(page_count: u32, shard_count: u32, error_message: String) -> Self {
        Self {
            page_count,
            shard_count,
            manifest_arrow_path: None,
            ocr_input_arrow_path: None,
            pending_resource_arrow_path: None,
            status: PdfRenderStatus::Fallback,
            routing_decision: PdfRenderRoutingDecision::FullDoclingFallback,
            error_message: Some(error_message),
        }
    }

    fn preflight_failed(error_message: String) -> Self {
        Self {
            page_count: 0,
            shard_count: 0,
            manifest_arrow_path: None,
            ocr_input_arrow_path: None,
            pending_resource_arrow_path: None,
            status: PdfRenderStatus::Fallback,
            routing_decision: PdfRenderRoutingDecision::PreflightFailed,
            error_message: Some(error_message),
        }
    }

    fn skipped(
        page_count: u32,
        routing_decision: PdfRenderRoutingDecision,
        error_message: String,
    ) -> Self {
        Self {
            page_count,
            shard_count: 0,
            manifest_arrow_path: None,
            ocr_input_arrow_path: None,
            pending_resource_arrow_path: None,
            status: PdfRenderStatus::Skipped,
            routing_decision,
            error_message: Some(error_message),
        }
    }

    fn rendered(
        page_count: u32,
        shard_count: u32,
        manifest_arrow_path: PathBuf,
        ocr_input_arrow_path: PathBuf,
        pending_resource_arrow_path: PathBuf,
    ) -> Self {
        Self {
            page_count,
            shard_count,
            manifest_arrow_path: Some(manifest_arrow_path),
            ocr_input_arrow_path: Some(ocr_input_arrow_path),
            pending_resource_arrow_path: Some(pending_resource_arrow_path),
            status: PdfRenderStatus::Rendered,
            routing_decision: PdfRenderRoutingDecision::HybridPageOcrCandidate,
            error_message: None,
        }
    }
}

fn render_document_manifests(
    document: &PdfDocument<'_>,
    context: &RenderShardContext<'_>,
    source_hash: &str,
    selected_page_indices: Option<&[i32]>,
) -> Result<Vec<PdfPageShardManifest>, ReportParts> {
    let page_count = u32::try_from(document.pages().len()).unwrap_or_default();
    let shard_dir = context.shard_dir(source_hash);
    fs::create_dir_all(shard_dir.as_path()).map_err(|error| {
        ReportParts::fallback(
            page_count,
            0,
            format!("create shard dir `{}`: {error}", shard_dir.display()),
        )
    })?;

    let mut manifests = Vec::new();
    let page_indices = selected_page_indices.map_or_else(
        || document.pages().as_range().collect::<Vec<_>>(),
        <[i32]>::to_vec,
    );
    for page_index in page_indices {
        let page = document.pages().get(page_index).map_err(|error| {
            ReportParts::fallback(
                page_count,
                checked_len_u32(manifests.len()),
                format!("load page {page_index}: {error}"),
            )
        })?;
        let manifest =
            render_page_manifest(&page, page_index, context, source_hash).map_err(|error| {
                ReportParts::fallback(page_count, checked_len_u32(manifests.len()), error)
            })?;
        manifests.push(manifest);
    }
    Ok(manifests)
}

fn render_document_region_manifests(
    document: &PdfDocument<'_>,
    context: &RenderShardContext<'_>,
    source_hash: &str,
    regions: &[PdfPageRegionRenderRequest],
) -> Result<Vec<PdfPageShardManifest>, ReportParts> {
    let page_count = u32::try_from(document.pages().len()).unwrap_or_default();
    let shard_dir = context.shard_dir(source_hash);
    fs::create_dir_all(shard_dir.as_path()).map_err(|error| {
        ReportParts::fallback(
            page_count,
            0,
            format!("create shard dir `{}`: {error}", shard_dir.display()),
        )
    })?;

    let mut sorted_regions = regions.to_vec();
    sorted_regions.sort_by(|left, right| {
        left.page_index
            .cmp(&right.page_index)
            .then_with(|| {
                left.effective_reading_order_key()
                    .cmp(&right.effective_reading_order_key())
            })
            .then_with(|| left.region_index.cmp(&right.region_index))
    });

    let mut manifests = Vec::new();
    let mut cursor = 0;
    while cursor < sorted_regions.len() {
        let page_index = sorted_regions[cursor].page_index;
        let next_cursor = sorted_regions[cursor..]
            .iter()
            .position(|region| region.page_index != page_index)
            .map_or(sorted_regions.len(), |offset| cursor + offset);
        let page = document
            .pages()
            .get(i32::try_from(page_index).unwrap_or(i32::MAX))
            .map_err(|error| {
                ReportParts::fallback(
                    page_count,
                    checked_len_u32(manifests.len()),
                    format!("load page {page_index}: {error}"),
                )
            })?;
        let page_manifests = render_page_region_manifests(
            &page,
            page_index,
            context,
            source_hash,
            &sorted_regions[cursor..next_cursor],
        )
        .map_err(|error| {
            ReportParts::fallback(page_count, checked_len_u32(manifests.len()), error)
        })?;
        manifests.extend(page_manifests);
        cursor = next_cursor;
    }
    Ok(manifests)
}

fn render_page_manifest(
    page: &PdfPage<'_>,
    page_index: i32,
    context: &RenderShardContext<'_>,
    source_hash: &str,
) -> Result<PdfPageShardManifest, String> {
    let rendered = render_page_image(page, page_index, context.profile)?;
    let image_path = context.shard_dir(source_hash).join(format!(
        "page-{page_index:05}.{}",
        context.profile.image_extension
    ));
    let raster = save_image_identity(&rendered.image, image_path.as_path())?;
    Ok(build_shard_manifest(PdfPageShardManifestInput {
        source_path: context.path,
        source_content_hash: source_hash,
        page_index: u32::try_from(page_index).unwrap_or_default(),
        profile: context.profile,
        media_box: rendered.media_box,
        crop_box: rendered.crop_box,
        rotation_degrees: rendered.rotation_degrees,
        raster,
    }))
}

fn render_page_region_manifests(
    page: &PdfPage<'_>,
    page_index: u32,
    context: &RenderShardContext<'_>,
    source_hash: &str,
    regions: &[PdfPageRegionRenderRequest],
) -> Result<Vec<PdfPageShardManifest>, String> {
    let rendered = render_page_image(
        page,
        i32::try_from(page_index).unwrap_or(i32::MAX),
        context.profile,
    )?;
    let parent_shard_element_id =
        shard_element_id(source_hash, page_index, context.profile.profile_id.as_str());
    regions
        .iter()
        .map(|request| {
            let source_page_pixel_box = region_pixel_box_for_crop(
                rendered.crop_box,
                request.region_box,
                rendered.image.width(),
                rendered.image.height(),
            )?;
            let image_path = context.shard_dir(source_hash).join(format!(
                "page-{page_index:05}-region-{:05}.{}",
                request.region_index, context.profile.image_extension
            ));
            let raster = save_region_crop_image(
                &rendered.image,
                source_page_pixel_box,
                image_path.as_path(),
            )?;
            build_region_shard_manifest(PdfPageRegionShardManifestInput {
                source_path: context.path,
                source_content_hash: source_hash,
                page_index,
                profile: context.profile,
                media_box: rendered.media_box,
                page_crop_box: rendered.crop_box,
                region: PdfPageRegion::new(
                    request.region_index,
                    request.region_box,
                    parent_shard_element_id.clone(),
                    request.effective_reading_order_key(),
                ),
                rotation_degrees: rendered.rotation_degrees,
                page_raster_width_px: rendered.image.width(),
                page_raster_height_px: rendered.image.height(),
                raster,
            })
        })
        .collect()
}

struct RenderedPageImage {
    image: DynamicImage,
    media_box: PdfPageBox,
    crop_box: PdfPageBox,
    rotation_degrees: u16,
}

fn render_page_image(
    page: &PdfPage<'_>,
    page_index: i32,
    profile: &PdfPageRenderProfile,
) -> Result<RenderedPageImage, String> {
    let media_box = page.boundaries().media().map_or_else(
        |_| PdfPageBox::from_pdfium_rect(page.page_size()),
        |boundary| PdfPageBox::from_pdfium_rect(boundary.bounds),
    );
    let crop_box = page.boundaries().crop().map_or(media_box, |boundary| {
        PdfPageBox::from_pdfium_rect(boundary.bounds)
    });
    let rotation_degrees = rotation_to_degrees(
        page.rotation()
            .map_err(|error| format!("read page {page_index} rotation: {error}"))?,
    );
    let (target_width, target_height) =
        render_dimensions_for_box(crop_box, rotation_degrees, profile);
    let config = PdfRenderConfig::new()
        .set_target_size(
            checked_pixels_i32(target_width)?,
            checked_pixels_i32(target_height)?,
        )
        .set_format(PdfBitmapFormat::BGRA)
        .render_annotations(profile.render_annotations)
        .render_form_data(profile.render_form_data);
    let bitmap = page
        .render_with_config(&config)
        .map_err(|error| format!("render page {page_index}: {error}"))?;
    let image = bitmap
        .as_image()
        .map_err(|error| format!("convert page {page_index} bitmap to image: {error}"))?;
    Ok(RenderedPageImage {
        image,
        media_box,
        crop_box,
        rotation_degrees,
    })
}

fn save_image_identity(
    image: &DynamicImage,
    image_path: &Path,
) -> Result<RenderedRasterIdentity, String> {
    image
        .save(image_path)
        .map_err(|error| format!("write shard image `{}`: {error}", image_path.display()))?;
    let raster_bytes = fs::read(image_path)
        .map_err(|error| format!("read shard image `{}`: {error}", image_path.display()))?;
    Ok(RenderedRasterIdentity {
        path: image_path.to_path_buf(),
        sha256: sha256_hex(&raster_bytes),
        width_px: image.width(),
        height_px: image.height(),
    })
}

fn save_region_crop_image(
    page_image: &DynamicImage,
    pixel_box: PdfPagePixelBox,
    image_path: &Path,
) -> Result<RenderedRasterIdentity, String> {
    if pixel_box.right > page_image.width() || pixel_box.bottom > page_image.height() {
        return Err(format!(
            "region pixel box exceeds source raster: box=({}, {}, {}, {}), raster={}x{}",
            pixel_box.left,
            pixel_box.top,
            pixel_box.right,
            pixel_box.bottom,
            page_image.width(),
            page_image.height()
        ));
    }
    let crop = page_image.crop_imm(
        pixel_box.left,
        pixel_box.top,
        pixel_box.width_px(),
        pixel_box.height_px(),
    );
    save_image_identity(&crop, image_path)
}

fn write_shard_artifact_batches(
    output_dir: &Path,
    manifests: &[PdfPageShardManifest],
    manifest_batch: RecordBatch,
) -> Result<(PathBuf, PathBuf, PathBuf), String> {
    let ocr_inputs = build_ocr_shard_inputs(manifests, &PdfOcrWorkerProfile::docling_compatible());
    let ocr_input_batch = build_ocr_shard_input_batch(&ocr_inputs)?;
    let pending_batch = build_ocr_pending_resource_batch(manifests)?;
    let manifest_arrow_path = output_dir.join(OCR_SHARD_MANIFEST_ARROW_NAME);
    let ocr_input_arrow_path = output_dir.join(OCR_SHARD_INPUT_ARROW_NAME);
    let pending_resource_arrow_path = output_dir.join(OCR_PENDING_RESOURCE_ARROW_NAME);
    write_arrow_file(manifest_arrow_path.as_path(), &[manifest_batch])?;
    write_arrow_file(ocr_input_arrow_path.as_path(), &[ocr_input_batch])?;
    write_arrow_file(pending_resource_arrow_path.as_path(), &[pending_batch])?;
    Ok((
        manifest_arrow_path,
        ocr_input_arrow_path,
        pending_resource_arrow_path,
    ))
}

fn shard_manifest_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("sourcePath", DataType::Utf8, false),
        Field::new("sourceContentHash", DataType::Utf8, false),
        Field::new("pageIndex", DataType::Int32, false),
        Field::new("renderProfile", DataType::Utf8, false),
        Field::new("imagePath", DataType::Utf8, false),
        Field::new("imageMimeType", DataType::Utf8, false),
        Field::new("rasterSha256", DataType::Utf8, false),
        Field::new("rasterWidthPx", DataType::Int32, false),
        Field::new("rasterHeightPx", DataType::Int32, false),
        Field::new("renderDpi", DataType::Int32, false),
        Field::new("rotationDegrees", DataType::Int32, false),
        Field::new("mediaLeft", DataType::Float64, false),
        Field::new("mediaBottom", DataType::Float64, false),
        Field::new("mediaRight", DataType::Float64, false),
        Field::new("mediaTop", DataType::Float64, false),
        Field::new("cropLeft", DataType::Float64, false),
        Field::new("cropBottom", DataType::Float64, false),
        Field::new("cropRight", DataType::Float64, false),
        Field::new("cropTop", DataType::Float64, false),
        Field::new("pointToPixelScaleX", DataType::Float64, false),
        Field::new("pointToPixelScaleY", DataType::Float64, false),
        Field::new("elementId", DataType::Utf8, false),
        Field::new("shardType", DataType::Utf8, false),
        Field::new("regionIndex", DataType::Int32, false),
        Field::new("parentShardElementId", DataType::Utf8, false),
        Field::new("readingOrderKey", DataType::Utf8, false),
        Field::new("sourcePagePixelLeft", DataType::Int32, false),
        Field::new("sourcePagePixelTop", DataType::Int32, false),
        Field::new("sourcePagePixelRight", DataType::Int32, false),
        Field::new("sourcePagePixelBottom", DataType::Int32, false),
    ]))
}

fn document_resource_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("sourcePath", DataType::Utf8, true),
        Field::new("resourceType", DataType::Utf8, true),
        Field::new("resourcePath", DataType::Utf8, true),
        Field::new("pageIndex", DataType::Int32, true),
        Field::new("caption", DataType::Utf8, true),
        Field::new("content", DataType::Utf8, true),
        Field::new("mimeType", DataType::Utf8, true),
        Field::new("status", DataType::Utf8, true),
        Field::new("elementId", DataType::Utf8, true),
    ]))
}

fn write_arrow_file(path: &Path, batches: &[RecordBatch]) -> Result<(), String> {
    if batches.is_empty() {
        return Err(format!(
            "cannot write empty Arrow IPC file `{}`",
            path.display()
        ));
    }
    let file = File::create(path)
        .map_err(|error| format!("create Arrow IPC file `{}`: {error}", path.display()))?;
    let mut writer = FileWriter::try_new(file, batches[0].schema().as_ref())
        .map_err(|error| format!("create Arrow IPC writer `{}`: {error}", path.display()))?;
    for batch in batches {
        writer
            .write(batch)
            .map_err(|error| format!("write Arrow IPC batch `{}`: {error}", path.display()))?;
    }
    writer
        .finish()
        .map_err(|error| format!("finish Arrow IPC file `{}`: {error}", path.display()))
}

fn render_markdown_report(records: &[PdfPageRenderShardReport]) -> String {
    let mut markdown = String::new();
    markdown.push_str("# PDF Page Render Shard Manifest Report\n\n");
    markdown.push_str("| Source | Status | Decision | Pages | Shards | Elapsed ms | Error |\n");
    markdown.push_str("| ------ | ------ | -------- | ----: | -----: | ---------: | ----- |\n");
    for record in records {
        let _ = writeln!(
            markdown,
            "| `{}` | `{}` | `{}` | {} | {} | {:.3} | {} |",
            record.source_path,
            record.status,
            record.routing_decision,
            record.page_count,
            record.shard_count,
            record.elapsed_ms,
            record.error_message.as_deref().unwrap_or("")
        );
    }
    markdown
}

#[cfg(test)]
#[path = "../../tests/unit/pdf/render.rs"]
mod tests;
