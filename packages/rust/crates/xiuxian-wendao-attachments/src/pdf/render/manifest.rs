//! Manifest construction and point-to-pixel geometry helpers.

use num_traits::ToPrimitive;

use super::identity::{
    page_reading_order_key, points_to_pixels, region_shard_element_id, shard_element_id,
};
use super::types::{
    PdfOcrShardType, PdfPageBox, PdfPagePixelBox, PdfPageRegionShardManifestInput,
    PdfPageRenderProfile, PdfPageShardGeometry, PdfPageShardManifest, PdfPageShardManifestInput,
};

/// Compute raster dimensions for a PDF point box and render profile.
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
/// Build a stable page shard manifest from render inputs.
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
    f64_to_u32_saturating(value.floor().clamp(0.0, f64::from(max)))
}

fn ceil_pixel(value: f64, max: u32) -> u32 {
    f64_to_u32_saturating(value.ceil().clamp(0.0, f64::from(max)))
}

fn f64_to_u32_saturating(value: f64) -> u32 {
    if !value.is_finite() || value <= 0.0 {
        return 0;
    }
    value.to_u32().unwrap_or(u32::MAX)
}
