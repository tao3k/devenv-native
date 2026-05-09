//! Image preprocessing and resize policy for vision requests.

use std::io::Cursor;
use std::sync::Arc;

use image::GenericImageView;
use image::imageops::FilterType;
use imageproc::contrast::equalize_histogram;

use crate::llm::error::{LlmError, LlmResult};

/// Default longest-edge bound used by vision preprocessing.
pub const DEFAULT_VISION_MAX_DIMENSION: u32 = 2_048;

/// Strategy used to prepare an image for downstream vision/OCR stages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreparedVisionImageMode {
    /// The image was resized and re-encoded into OCR-friendly PNG artifacts.
    Preprocessed,
    /// The original image bytes are passed through to the OCR engine.
    OriginalPassthrough,
}

impl PreparedVisionImageMode {
    /// Returns a stable label for telemetry and tests.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Preprocessed => "preprocessed",
            Self::OriginalPassthrough => "original_passthrough",
        }
    }
}

/// Preprocessed image payloads ready for multimodal/vision pipelines.
#[derive(Debug, Clone)]
pub struct PreparedVisionImage {
    /// Strategy used to build this prepared image payload.
    pub mode: PreparedVisionImageMode,
    /// Original input bytes.
    pub original: Arc<[u8]>,
    /// Bytes forwarded into the OCR engine image decoder.
    pub engine_input: Arc<[u8]>,
    /// Resized RGB payload encoded as PNG.
    pub resized_png: Arc<[u8]>,
    /// Equalized grayscale payload encoded as PNG.
    pub grayscale_png: Arc<[u8]>,
    /// Final image width after preprocessing.
    pub width: u32,
    /// Final image height after preprocessing.
    pub height: u32,
    /// Scale factor applied to the original dimensions.
    pub scale: f64,
}

impl PreparedVisionImage {
    /// Create a dummy prepared image for prewarm/testing.
    #[must_use]
    pub fn create_dummy(width: u32, height: u32) -> Self {
        let dummy_data: Arc<[u8]> = Arc::from(vec![0u8; 0]);
        Self {
            mode: PreparedVisionImageMode::Preprocessed,
            original: dummy_data.clone(),
            engine_input: dummy_data.clone(),
            resized_png: dummy_data.clone(),
            grayscale_png: dummy_data,
            width,
            height,
            scale: 1.0,
        }
    }
}

/// Preprocess an image using the default max dimension bound.
///
/// # Errors
///
/// Returns an error when decoding or PNG encoding fails.
pub fn preprocess_image(image_bytes: Arc<[u8]>) -> LlmResult<PreparedVisionImage> {
    preprocess_image_with_max_dimension(image_bytes, DEFAULT_VISION_MAX_DIMENSION)
}

/// Prepare an OCR image by preserving the original bytes as the engine input.
///
/// This matches the upstream CLI flow more closely: decode the original bytes
/// only to discover image dimensions, and let the model pipeline own the
/// actual vision preprocessing.
///
/// # Errors
///
/// Returns an error when the source image cannot be decoded.
pub fn prepare_image_for_ocr_runtime(image_bytes: Arc<[u8]>) -> LlmResult<PreparedVisionImage> {
    let (width, height) = decode_image_dimensions(image_bytes.as_ref())?;
    let prepared = PreparedVisionImage {
        mode: PreparedVisionImageMode::OriginalPassthrough,
        original: Arc::clone(&image_bytes),
        engine_input: Arc::clone(&image_bytes),
        resized_png: Arc::clone(&image_bytes),
        grayscale_png: image_bytes,
        width,
        height,
        scale: 1.0,
    };
    tracing::info!(
        event = "llm.vision.deepseek.prepare_input",
        input_mode = prepared.mode.as_str(),
        width = prepared.width,
        height = prepared.height,
        original_bytes = prepared.original.len(),
        engine_input_bytes = prepared.engine_input.len(),
        "Prepared DeepSeek OCR input from original image bytes"
    );
    Ok(prepared)
}

/// Preprocess an image with a custom longest-edge bound.
///
/// Steps:
/// 1. Decode image bytes.
/// 2. Resize to fit within `max_dimension` while preserving aspect ratio.
/// 3. Produce an equalized grayscale variant for OCR-friendly contrast.
///
/// # Errors
///
/// Returns an error when decoding or PNG encoding fails.
pub fn preprocess_image_with_max_dimension(
    image_bytes: Arc<[u8]>,
    max_dimension: u32,
) -> LlmResult<PreparedVisionImage> {
    let decoded = decode_image(image_bytes.as_ref())?;
    let (original_width, original_height) = decoded.dimensions();
    let dimensions = fit_dimensions(original_width, original_height, max_dimension);

    let resized = if dimensions.width == original_width && dimensions.height == original_height {
        decoded
    } else {
        decoded.resize_exact(dimensions.width, dimensions.height, FilterType::Lanczos3)
    };

    let resized_png = encode_png(&resized)?;
    let grayscale = equalize_histogram(&resized.to_luma8());
    let grayscale_png = encode_png(&image::DynamicImage::ImageLuma8(grayscale))?;

    Ok(PreparedVisionImage {
        mode: PreparedVisionImageMode::Preprocessed,
        original: image_bytes,
        engine_input: Arc::clone(&resized_png),
        resized_png,
        grayscale_png,
        width: dimensions.width,
        height: dimensions.height,
        scale: dimensions.scale,
    })
}

/// Resized image dimensions and scale factor.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FittedDimensions {
    /// Target width.
    pub width: u32,
    /// Target height.
    pub height: u32,
    /// Applied scale factor.
    pub scale: f64,
}

/// Computes resized dimensions that fit within `max_dimension` while preserving
/// aspect ratio.
#[must_use]
pub fn fit_dimensions(width: u32, height: u32, max_dimension: u32) -> FittedDimensions {
    if width == 0 || height == 0 {
        return FittedDimensions {
            width: 1,
            height: 1,
            scale: 1.0,
        };
    }
    if max_dimension == 0 {
        return FittedDimensions {
            width,
            height,
            scale: 1.0,
        };
    }

    let long_edge = width.max(height);
    if long_edge <= max_dimension {
        return FittedDimensions {
            width,
            height,
            scale: 1.0,
        };
    }

    let scale = f64::from(max_dimension) / f64::from(long_edge);
    let target_width = fit_edge_with_rounding(width, long_edge, max_dimension);
    let target_height = fit_edge_with_rounding(height, long_edge, max_dimension);
    FittedDimensions {
        width: target_width,
        height: target_height,
        scale,
    }
}

#[inline]
#[must_use]
fn fit_edge_with_rounding(edge: u32, long_edge: u32, max_dimension: u32) -> u32 {
    let numerator = u64::from(edge) * u64::from(max_dimension);
    let denominator = u64::from(long_edge);
    let rounded = (numerator + (denominator / 2)) / denominator;
    let bounded = rounded.max(1).min(u64::from(u32::MAX));
    u32::try_from(bounded).unwrap_or(u32::MAX)
}

/// Encode a dynamic image to PNG bytes.
///
/// # Errors
///
/// Returns an error when image encoding fails.
pub fn encode_png(image: &image::DynamicImage) -> LlmResult<Arc<[u8]>> {
    let mut writer = Cursor::new(Vec::new());
    image
        .write_to(&mut writer, image::ImageFormat::Png)
        .map_err(|error| internal_error(format!("vision png encode failed: {error}")))?;
    Ok(Arc::from(writer.into_inner()))
}

fn decode_image(image_bytes: &[u8]) -> LlmResult<image::DynamicImage> {
    image::load_from_memory(image_bytes)
        .map_err(|error| internal_error(format!("vision image decode failed: {error}")))
}

fn decode_image_dimensions(image_bytes: &[u8]) -> LlmResult<(u32, u32)> {
    Ok(decode_image(image_bytes)?.dimensions())
}

fn internal_error(message: String) -> LlmError {
    LlmError::Internal { message }
}
