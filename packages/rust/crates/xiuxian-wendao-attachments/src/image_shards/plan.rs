//! Planner for standalone image attachment shards.

use std::fs;
use std::path::Path;

use sha2::{Digest, Sha256};

use crate::image_audit::audit_image_attachment;

use super::types::{
    ImageMimeType, ImageShardOptions, ImageShardPlan, ImageShardSpec, ImageTileBox,
};

const OUTPUT_PROFILE: &str = "lossless-png-v1";

/// Plan deterministic image tiles for a standalone image attachment.
///
/// # Errors
///
/// Returns an error when the source cannot be audited, dimensions are
/// unavailable, options are invalid, or source bytes cannot be read.
pub fn plan_image_shards(
    source_path: &Path,
    options: ImageShardOptions,
) -> Result<ImageShardPlan, String> {
    let options = options.validate()?;
    let audit = audit_image_attachment(source_path)?;
    let width = audit
        .width_px
        .ok_or_else(|| "image shard planning requires known source width".to_string())?;
    let height = audit
        .height_px
        .ok_or_else(|| "image shard planning requires known source height".to_string())?;
    let source_bytes = fs::read(source_path)
        .map_err(|error| format!("read image source `{}`: {error}", source_path.display()))?;
    let source_hash = sha256_hex(source_bytes.as_slice());
    let shards = plan_tiles(width, height, options, source_hash.as_str())?;

    Ok(ImageShardPlan {
        source_path: source_path.to_path_buf(),
        source_content_hash: source_hash,
        source_mime_type: ImageMimeType::try_from(audit.mime_type.as_str())?,
        source_width: width,
        source_height: height,
        output_mime_type: ImageMimeType::Png,
        options,
        shards,
    })
}

pub(super) fn output_profile() -> &'static str {
    OUTPUT_PROFILE
}

fn plan_tiles(
    source_width: u32,
    source_height: u32,
    options: ImageShardOptions,
    source_hash: &str,
) -> Result<Vec<ImageShardSpec>, String> {
    if source_width == 0 || source_height == 0 {
        return Err("image shard source dimensions must be positive".to_string());
    }
    let mut shards = Vec::new();
    let mut top = 0;
    let step_y = options.max_tile_height - options.overlap;
    let step_x = options.max_tile_width - options.overlap;
    while top < source_height {
        let mut left = 0;
        let tile_height = options.max_tile_height.min(source_height - top);
        while left < source_width {
            let tile_width = options.max_tile_width.min(source_width - left);
            let tile_box = ImageTileBox::new(left, top, tile_width, tile_height)?;
            let shard_index = u32::try_from(shards.len())
                .map_err(|_| "image shard count exceeds u32".to_string())?;
            shards.push(ImageShardSpec {
                shard_index,
                tile_box,
                reading_order_key: format!("{shard_index:06}"),
                shard_digest: shard_digest(source_hash, tile_box),
            });
            if tile_box.right_px() >= source_width {
                break;
            }
            left = left.saturating_add(step_x);
        }
        if top + tile_height >= source_height {
            break;
        }
        top = top.saturating_add(step_y);
    }
    Ok(shards)
}

fn shard_digest(source_hash: &str, tile_box: ImageTileBox) -> String {
    let mut hasher = Sha256::new();
    hasher.update(source_hash.as_bytes());
    hasher.update(b":");
    hasher.update(output_profile().as_bytes());
    hasher.update(b":");
    hasher.update(tile_box.left.to_le_bytes());
    hasher.update(tile_box.top.to_le_bytes());
    hasher.update(tile_box.width.to_le_bytes());
    hasher.update(tile_box.height.to_le_bytes());
    hex_digest(hasher.finalize().as_slice())
}

pub(super) fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex_digest(hasher.finalize().as_slice())
}

fn hex_digest(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write as _;
        let _ = write!(output, "{byte:02x}");
    }
    output
}
