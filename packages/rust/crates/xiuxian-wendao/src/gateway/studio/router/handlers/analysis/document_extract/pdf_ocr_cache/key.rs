use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use sha2::{Digest, Sha256};
use xiuxian_wendao_attachments::pdf::ocr::PdfOcrShardInput;

pub(in super::super) fn ocr_shard_cache_key(input: &PdfOcrShardInput) -> String {
    let mut hasher = Sha256::new();
    for fragment in ocr_shard_cache_fragments(input) {
        hasher.update(fragment.as_bytes());
        hasher.update([0]);
    }
    format!("{:x}", hasher.finalize())
}

pub(super) fn temporary_cache_path(path: &Path) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("ocr-shard.arrow");
    path.with_file_name(format!(".{file_name}.{}.{}.tmp", std::process::id(), nanos))
}

fn ocr_shard_cache_fragments(input: &PdfOcrShardInput) -> Vec<String> {
    vec![
        input.contract_version.clone(),
        input.source_path.clone(),
        input.source_content_hash.clone(),
        input.page_index.to_string(),
        input.shard_type.clone(),
        input.region_index.to_string(),
        input.parent_shard_element_id.clone(),
        input.reading_order_key.clone(),
        input.image_mime_type.clone(),
        input.raster_sha256.clone(),
        input.render_profile.clone(),
        input.ocr_profile.clone(),
        input.ocr_engine.clone(),
        input.preferred_languages.join("\u{1f}"),
        f64_bits(input.min_confidence),
        input.preserve_layout.to_string(),
        input.raster_width_px.to_string(),
        input.raster_height_px.to_string(),
        input.render_dpi.to_string(),
        input.rotation_degrees.to_string(),
        f64_bits(input.crop_left),
        f64_bits(input.crop_bottom),
        f64_bits(input.crop_right),
        f64_bits(input.crop_top),
        f64_bits(input.point_to_pixel_scale_x),
        f64_bits(input.point_to_pixel_scale_y),
        input.source_page_pixel_left.to_string(),
        input.source_page_pixel_top.to_string(),
        input.source_page_pixel_right.to_string(),
        input.source_page_pixel_bottom.to_string(),
        input.shard_element_id.clone(),
    ]
}

fn f64_bits(value: f64) -> String {
    format!("{:016x}", value.to_bits())
}
