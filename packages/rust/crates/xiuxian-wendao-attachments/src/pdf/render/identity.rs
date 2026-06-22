//! Stable render identities, hashing, and scalar conversion helpers.

use std::path::Path;

#[cfg(feature = "pdf-render")]
use pdfium_render::prelude::PdfPageRenderRotation;
use sha2::{Digest, Sha256};

use num_traits::ToPrimitive;

use super::types::{PdfPageBox, PdfPageRenderProfile};

pub(super) const PDF_RENDER_SHARD_PROFILE: &str = "pdfium-render-page-shards-v1";
const PDF_SOURCE_PAGE_RANGE_PROFILE: &str = "source-pdf-page-range-shards-v1";

pub(super) fn points_to_pixels(points: f64, dpi: u32) -> u32 {
    f64_to_u32_saturating(((points / 72.0) * f64::from(dpi)).round().max(1.0))
}

#[cfg(feature = "pdf-render")]
pub(super) fn rotation_to_degrees(rotation: PdfPageRenderRotation) -> u16 {
    match rotation {
        PdfPageRenderRotation::None => 0,
        PdfPageRenderRotation::Degrees90 => 90,
        PdfPageRenderRotation::Degrees180 => 180,
        PdfPageRenderRotation::Degrees270 => 270,
    }
}

pub(super) fn shard_element_id(content_hash: &str, page_index: u32, profile_id: &str) -> String {
    sha256_hex(format!("{content_hash}:{page_index}:{profile_id}").as_bytes())
}

pub(super) fn region_shard_element_id(
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

pub(super) fn page_reading_order_key(page_index: u32) -> String {
    format!("{page_index:06}.000000")
}

fn f64_to_u32_saturating(value: f64) -> u32 {
    if !value.is_finite() || value <= 0.0 {
        return 0;
    }
    value.to_u32().unwrap_or(u32::MAX)
}

pub(crate) fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

pub(super) fn checked_len_u32(len: usize) -> u32 {
    u32::try_from(len).unwrap_or(u32::MAX)
}

#[cfg(feature = "pdf-render")]
pub(super) fn checked_pixels_i32(value: u32) -> Result<i32, String> {
    i32::try_from(value).map_err(|_| format!("render target pixel dimension is too large: {value}"))
}
pub(super) fn is_pdf_path(path: &Path) -> bool {
    path.extension()
        .and_then(|suffix| suffix.to_str())
        .is_some_and(|suffix| suffix.eq_ignore_ascii_case("pdf"))
}
pub(super) fn source_page_range_profile(profile: &PdfPageRenderProfile) -> PdfPageRenderProfile {
    let mut source_profile = profile.clone();
    source_profile.profile_id = PDF_SOURCE_PAGE_RANGE_PROFILE.to_string();
    source_profile.image_extension = "source-page-range".to_string();
    source_profile.image_mime_type = "application/x-wendao-source-pdf-page".to_string();
    source_profile
}
