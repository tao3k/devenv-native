//! OCR shard public data contracts and result constructors.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Stable schema version for OCR worker input batches.
pub const PDF_OCR_SHARD_INPUT_SCHEMA_VERSION: &str = "xiuxian_wendao.pdf_ocr_shard_input.v1";
/// Stable schema version for OCR worker result batches.
pub const PDF_OCR_SHARD_RESULT_SCHEMA_VERSION: &str = "xiuxian_wendao.pdf_ocr_shard_result.v1";
/// Default OCR worker profile identifier for Docling-compatible OCR.
pub const PDF_OCR_DEFAULT_PROFILE: &str = "docling-compatible-page-ocr-v1";
/// Fast Docling OCR worker profile identifier for low-risk source-range OCR.
pub const PDF_OCR_FAST_TEXT_PROFILE: &str = "docling-fast-text-ocr";
/// Direct DeepSeek-OCR-2 VLM worker profile identifier.
pub const PDF_OCR_DEEPSEEK_OCR2_DIRECT_VLM_PROFILE: &str = "deepseek-ocr2-direct-vlm";
/// Hosted OpenAI-compatible VLM/OCR worker profile identifier.
pub const PDF_OCR_HOSTED_VLM_DIRECT_PROFILE: &str = "hosted-vlm-direct-ocr-v1";
/// Docling VLM adapter profile identifier for `DeepSeek` OCR comparator runs.
pub const PDF_OCR_DOCLING_VLM_DEEPSEEK_OCR_PROFILE: &str = "docling-vlm-deepseek-ocr";

/// Return true when an OCR profile uses the hosted direct VLM recovery path.
#[must_use]
pub fn is_hosted_vlm_direct_profile(profile: &str) -> bool {
    matches!(
        profile,
        PDF_OCR_HOSTED_VLM_DIRECT_PROFILE | PDF_OCR_DEEPSEEK_OCR2_DIRECT_VLM_PROFILE
    )
}

/// OCR worker profile used to derive shard input rows.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfOcrWorkerProfile {
    pub profile_id: String,
    pub engine: String,
    pub preferred_languages: Vec<String>,
    pub min_confidence: f64,
    pub preserve_layout: bool,
}

impl PdfOcrWorkerProfile {
    /// Return the default Docling-compatible OCR profile.
    #[must_use]
    pub fn docling_compatible() -> Self {
        Self {
            profile_id: PDF_OCR_DEFAULT_PROFILE.to_string(),
            engine: "docling-compatible-ocr".to_string(),
            preferred_languages: vec!["auto".to_string()],
            min_confidence: 0.0,
            preserve_layout: true,
        }
    }
}

impl Default for PdfOcrWorkerProfile {
    fn default() -> Self {
        Self::docling_compatible()
    }
}

/// One OCR worker input row for a rendered page or region shard.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfOcrShardInput {
    pub contract_version: String,
    pub source_path: String,
    pub source_content_hash: String,
    pub page_index: u32,
    pub image_path: String,
    pub image_mime_type: String,
    pub raster_sha256: String,
    pub render_profile: String,
    pub ocr_profile: String,
    pub ocr_engine: String,
    pub preferred_languages: Vec<String>,
    pub min_confidence: f64,
    pub preserve_layout: bool,
    pub raster_width_px: u32,
    pub raster_height_px: u32,
    pub render_dpi: u32,
    pub rotation_degrees: u16,
    pub crop_left: f64,
    pub crop_bottom: f64,
    pub crop_right: f64,
    pub crop_top: f64,
    pub point_to_pixel_scale_x: f64,
    pub point_to_pixel_scale_y: f64,
    pub shard_element_id: String,
    pub shard_type: String,
    pub region_index: u32,
    pub parent_shard_element_id: String,
    pub reading_order_key: String,
    pub source_page_pixel_left: u32,
    pub source_page_pixel_top: u32,
    pub source_page_pixel_right: u32,
    pub source_page_pixel_bottom: u32,
}

/// Source contract required to derive OCR worker input rows from render shards.
pub trait OcrShardManifestSource {
    /// Source document path.
    fn source_path(&self) -> &str;
    /// Source document content hash.
    fn source_content_hash(&self) -> &str;
    /// Zero-based page index.
    fn page_index(&self) -> u32;
    /// Rendered shard image path.
    fn image_path(&self) -> &str;
    /// Rendered shard image MIME type.
    fn image_mime_type(&self) -> &str;
    /// Rendered raster SHA-256 digest.
    fn raster_sha256(&self) -> &str;
    /// Render profile identifier.
    fn render_profile(&self) -> &str;
    /// Rendered raster width in pixels.
    fn raster_width_px(&self) -> u32;
    /// Rendered raster height in pixels.
    fn raster_height_px(&self) -> u32;
    /// Render DPI.
    fn render_dpi(&self) -> u32;
    /// Page rotation in degrees.
    fn rotation_degrees(&self) -> u16;
    /// Crop left coordinate in PDF points.
    fn crop_left(&self) -> f64;
    /// Crop bottom coordinate in PDF points.
    fn crop_bottom(&self) -> f64;
    /// Crop right coordinate in PDF points.
    fn crop_right(&self) -> f64;
    /// Crop top coordinate in PDF points.
    fn crop_top(&self) -> f64;
    /// Point-to-pixel scale on the x axis.
    fn point_to_pixel_scale_x(&self) -> f64;
    /// Point-to-pixel scale on the y axis.
    fn point_to_pixel_scale_y(&self) -> f64;
    /// Stable shard element identifier.
    fn shard_element_id(&self) -> &str;
    /// Stable shard type string.
    fn shard_type(&self) -> &str;
    /// Region index for region shards, or zero for page shards.
    fn region_index(&self) -> u32;
    /// Parent shard element identifier for region shards.
    fn parent_shard_element_id(&self) -> &str;
    /// Stable reading order key.
    fn reading_order_key(&self) -> &str;
    /// Source page pixel left bound.
    fn source_page_pixel_left(&self) -> u32;
    /// Source page pixel top bound.
    fn source_page_pixel_top(&self) -> u32;
    /// Source page pixel right bound.
    fn source_page_pixel_right(&self) -> u32;
    /// Source page pixel bottom bound.
    fn source_page_pixel_bottom(&self) -> u32;
}

/// Stable OCR worker result status values.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PdfOcrShardResultStatus {
    Succeeded,
    Failed,
    Skipped,
}

impl PdfOcrShardResultStatus {
    /// Return the stable serialized status string.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
            Self::Skipped => "skipped",
        }
    }

    /// Decode a stable OCR result status value.
    ///
    /// # Errors
    ///
    /// Returns an error when the status is outside the stable result contract.
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "succeeded" => Ok(Self::Succeeded),
            "failed" => Ok(Self::Failed),
            "skipped" => Ok(Self::Skipped),
            other => Err(format!("unsupported OCR shard result status `{other}`")),
        }
    }
}

/// One OCR worker result row for a rendered page or region shard.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfOcrShardResult {
    pub contract_version: String,
    pub source_path: String,
    pub source_content_hash: String,
    pub page_index: u32,
    pub image_path: String,
    pub image_mime_type: String,
    pub raster_sha256: String,
    pub render_profile: String,
    pub ocr_profile: String,
    pub status: PdfOcrShardResultStatus,
    pub text: Option<String>,
    pub text_mime_type: String,
    pub confidence: Option<f64>,
    pub error_message: Option<String>,
    pub shard_element_id: String,
    pub element_id: String,
}

impl PdfOcrShardResult {
    /// Build a successful OCR result for an input shard.
    #[must_use]
    pub fn succeeded(input: &PdfOcrShardInput, text: impl Into<String>, confidence: f64) -> Self {
        Self::from_input(
            input,
            PdfOcrShardResultStatus::Succeeded,
            Some(text.into()),
            Some(confidence),
            None,
        )
    }

    /// Build a failed OCR result for an input shard.
    #[must_use]
    pub fn failed(input: &PdfOcrShardInput, error_message: impl Into<String>) -> Self {
        Self::from_input(
            input,
            PdfOcrShardResultStatus::Failed,
            None,
            None,
            Some(error_message.into()),
        )
    }

    /// Build a skipped OCR result for an input shard.
    #[must_use]
    pub fn skipped(input: &PdfOcrShardInput, reason: impl Into<String>) -> Self {
        Self::from_input(
            input,
            PdfOcrShardResultStatus::Skipped,
            None,
            None,
            Some(reason.into()),
        )
    }

    fn from_input(
        input: &PdfOcrShardInput,
        status: PdfOcrShardResultStatus,
        text: Option<String>,
        confidence: Option<f64>,
        error_message: Option<String>,
    ) -> Self {
        Self {
            contract_version: PDF_OCR_SHARD_RESULT_SCHEMA_VERSION.to_string(),
            source_path: input.source_path.clone(),
            source_content_hash: input.source_content_hash.clone(),
            page_index: input.page_index,
            image_path: input.image_path.clone(),
            image_mime_type: input.image_mime_type.clone(),
            raster_sha256: input.raster_sha256.clone(),
            render_profile: input.render_profile.clone(),
            ocr_profile: input.ocr_profile.clone(),
            status,
            text,
            text_mime_type: "text/plain".to_string(),
            confidence,
            error_message,
            shard_element_id: input.shard_element_id.clone(),
            element_id: ocr_result_element_id(input),
        }
    }
}

fn ocr_result_element_id(input: &PdfOcrShardInput) -> String {
    sha256_hex(
        format!(
            "{}:{}:{}:{}:{}:{}",
            input.source_content_hash,
            input.page_index,
            input.render_profile,
            input.ocr_profile,
            input.shard_element_id,
            input.raster_sha256
        )
        .as_bytes(),
    )
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}
