//! Public PDF render shard data contracts.

use std::path::{Path, PathBuf};

#[cfg(feature = "pdf-render")]
use pdfium_render::prelude::PdfRect;
use serde::{Deserialize, Serialize};

use crate::pdf::ocr::OcrShardManifestSource;

const PDF_RENDER_SHARD_PROFILE: &str = "pdfium-render-page-shards-v1";

/// Routing decision for PDF rendering acceleration.
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
    /// Return the stable serialized routing decision string.
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

/// Status of a PDF render shard operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PdfRenderStatus {
    Rendered,
    Fallback,
    Skipped,
    Unsupported,
}

impl PdfRenderStatus {
    /// Return the stable serialized render status string.
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

/// Page selection mode for PDF shard rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PdfPageRenderSelection {
    AllPages,
    ShardFallbackPages,
    RegionShards,
}

impl PdfPageRenderSelection {
    /// Return the stable serialized page selection string.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::AllPages => "all_pages",
            Self::ShardFallbackPages => "shard_fallback_pages",
            Self::RegionShards => "region_shards",
        }
    }
}

/// Stringly state boundary for PDF page render profiles.
///
/// The image MIME type is a serialized worker-profile token shared with Arrow
/// sidecars and Python OCR workers.
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
    /// Return the default OCR-oriented render profile.
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

/// PDF page box in point coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfPageBox {
    pub left: f64,
    pub bottom: f64,
    pub right: f64,
    pub top: f64,
}

impl PdfPageBox {
    /// Create a normalized PDF page box.
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

    /// Return the box width in PDF points.
    #[must_use]
    pub fn width_points(&self) -> f64 {
        self.right - self.left
    }

    /// Return the box height in PDF points.
    #[must_use]
    pub fn height_points(&self) -> f64 {
        self.top - self.bottom
    }

    /// Return the geometric intersection of two PDF point boxes.
    #[must_use]
    pub fn intersection(&self, other: Self) -> Option<Self> {
        let left = self.left.max(other.left);
        let bottom = self.bottom.max(other.bottom);
        let right = self.right.min(other.right);
        let top = self.top.min(other.top);
        (left < right && bottom < top).then(|| Self::new(left, bottom, right, top))
    }

    #[cfg(feature = "pdf-render")]
    pub(super) fn from_pdfium_rect(rect: PdfRect) -> Self {
        Self::new(
            f64::from(rect.left().value),
            f64::from(rect.bottom().value),
            f64::from(rect.right().value),
            f64::from(rect.top().value),
        )
    }
}

/// OCR shard kind for a full page or cropped region.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PdfOcrShardType {
    Page,
    Region,
}

impl PdfOcrShardType {
    /// Return the stable serialized shard type string.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Page => "page",
            Self::Region => "region",
        }
    }
}

/// Pixel-space box within a rendered page raster.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfPagePixelBox {
    pub left: u32,
    pub top: u32,
    pub right: u32,
    pub bottom: u32,
}

impl PdfPagePixelBox {
    /// Create a pixel-space box.
    #[must_use]
    pub fn new(left: u32, top: u32, right: u32, bottom: u32) -> Self {
        Self {
            left,
            top,
            right,
            bottom,
        }
    }

    /// Return the pixel width.
    #[must_use]
    pub fn width_px(&self) -> u32 {
        self.right.saturating_sub(self.left)
    }

    /// Return the pixel height.
    #[must_use]
    pub fn height_px(&self) -> u32 {
        self.bottom.saturating_sub(self.top)
    }
}

/// Region crop request tied to a parent page shard.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfPageRegion {
    pub region_index: u32,
    pub region_box: PdfPageBox,
    pub parent_shard_element_id: String,
    pub reading_order_key: String,
}

impl PdfPageRegion {
    /// Create a page region crop descriptor.
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

/// Geometry attached to a rendered PDF page or region shard.
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

/// Raw DTO boundary and stringly state boundary for rendered PDF shard rows.
///
/// The manifest is a stable Arrow/JSON sidecar contract, so source paths,
/// image paths, MIME type, and element identifiers remain primitive serialized
/// fields at this transport boundary.
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

impl OcrShardManifestSource for PdfPageShardManifest {
    fn source_path(&self) -> &str {
        self.source_path.as_str()
    }

    fn source_content_hash(&self) -> &str {
        self.source_content_hash.as_str()
    }

    fn page_index(&self) -> u32 {
        self.page_index
    }

    fn image_path(&self) -> &str {
        self.image_path.as_str()
    }

    fn image_mime_type(&self) -> &str {
        self.image_mime_type.as_str()
    }

    fn raster_sha256(&self) -> &str {
        self.raster_sha256.as_str()
    }

    fn render_profile(&self) -> &str {
        self.render_profile.as_str()
    }

    fn raster_width_px(&self) -> u32 {
        self.geometry.raster_width_px
    }

    fn raster_height_px(&self) -> u32 {
        self.geometry.raster_height_px
    }

    fn render_dpi(&self) -> u32 {
        self.geometry.render_dpi
    }

    fn rotation_degrees(&self) -> u16 {
        self.geometry.rotation_degrees
    }

    fn crop_left(&self) -> f64 {
        self.geometry.crop_box.left
    }

    fn crop_bottom(&self) -> f64 {
        self.geometry.crop_box.bottom
    }

    fn crop_right(&self) -> f64 {
        self.geometry.crop_box.right
    }

    fn crop_top(&self) -> f64 {
        self.geometry.crop_box.top
    }

    fn point_to_pixel_scale_x(&self) -> f64 {
        self.geometry.point_to_pixel_scale_x
    }

    fn point_to_pixel_scale_y(&self) -> f64 {
        self.geometry.point_to_pixel_scale_y
    }

    fn shard_element_id(&self) -> &str {
        self.element_id.as_str()
    }

    fn shard_type(&self) -> &str {
        self.shard_type.as_str()
    }

    fn region_index(&self) -> u32 {
        self.region_index
    }

    fn parent_shard_element_id(&self) -> &str {
        self.parent_shard_element_id.as_str()
    }

    fn reading_order_key(&self) -> &str {
        self.reading_order_key.as_str()
    }

    fn source_page_pixel_left(&self) -> u32 {
        self.source_page_pixel_box.left
    }

    fn source_page_pixel_top(&self) -> u32 {
        self.source_page_pixel_box.top
    }

    fn source_page_pixel_right(&self) -> u32 {
        self.source_page_pixel_box.right
    }

    fn source_page_pixel_bottom(&self) -> u32 {
        self.source_page_pixel_box.bottom
    }
}

/// Raw DTO boundary and stringly state boundary for PDF render reports.
///
/// This report is serialized for diagnostics and cache receipts, so output
/// paths and status/routing tokens remain primitive fields.
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
/// Input data needed to build a page shard manifest.
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

/// Input data needed to build a region shard manifest.
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

/// Request to render one region from a source PDF page.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfPageRegionRenderRequest {
    pub page_index: u32,
    pub region_index: u32,
    pub region_box: PdfPageBox,
    pub reading_order_key: Option<String>,
}

impl PdfPageRegionRenderRequest {
    /// Create a region render request.
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

    /// Return the explicit reading-order key or the deterministic page/region fallback.
    #[must_use]
    pub fn effective_reading_order_key(&self) -> String {
        self.reading_order_key
            .clone()
            .unwrap_or_else(|| format!("{:06}.{:06}", self.page_index, self.region_index))
    }
}
/// Identity of a rendered raster artifact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderedRasterIdentity {
    pub path: PathBuf,
    pub sha256: String,
    pub width_px: u32,
    pub height_px: u32,
}
