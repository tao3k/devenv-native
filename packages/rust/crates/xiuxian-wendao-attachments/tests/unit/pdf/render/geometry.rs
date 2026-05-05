use super::{
    assert_close, sample_manifest, sample_region_manifest, save_region_crop_image, shard_element_id,
};
use crate::pdf::render::{
    PdfOcrShardType, PdfPageBox, PdfPagePixelBox, PdfPageRegionRenderRequest, PdfPageRenderProfile,
    region_pixel_box_for_crop, render_dimensions_for_box,
};
use image::{DynamicImage, ImageBuffer, Rgba};

#[test]
fn document_extract_pdf_render_dimensions_follow_dpi_and_rotation() {
    let profile = PdfPageRenderProfile::ocr_default();
    let page_box = PdfPageBox::new(0.0, 0.0, 612.0, 792.0);

    assert_eq!(
        render_dimensions_for_box(page_box, 0, &profile),
        (2550, 3300)
    );
    assert_eq!(
        render_dimensions_for_box(page_box, 90, &profile),
        (3300, 2550)
    );
    assert_eq!(
        render_dimensions_for_box(page_box, 270, &profile),
        (3300, 2550)
    );
}

#[test]
fn document_extract_pdf_render_manifest_preserves_boxes_and_transform() {
    let manifest = sample_manifest(90);

    assert_close(manifest.geometry.media_box.width_points(), 612.0);
    assert_close(manifest.geometry.crop_box.left, 18.0);
    assert_eq!(manifest.shard_type, PdfOcrShardType::Page);
    assert_eq!(manifest.region_index, 0);
    assert_eq!(manifest.reading_order_key, "000002.000000");
    assert_eq!(manifest.source_page_pixel_box.width_px(), 2400);
    assert_eq!(manifest.source_page_pixel_box.height_px(), 3100);
    assert_eq!(manifest.geometry.rotation_degrees, 90);
    assert_eq!(manifest.geometry.render_dpi, 300);
    assert!(manifest.geometry.point_to_pixel_scale_x > 4.0);
    assert!(manifest.geometry.point_to_pixel_scale_y > 4.0);
    assert_eq!(manifest.image_mime_type, "image/png");
}

#[test]
fn document_extract_pdf_render_maps_region_to_source_page_pixels() -> Result<(), String> {
    let pixel_box = region_pixel_box_for_crop(
        PdfPageBox::new(18.0, 24.0, 594.0, 768.0),
        PdfPageBox::new(162.0, 210.0, 306.0, 396.0),
        2400,
        3100,
    )?;

    assert_eq!(pixel_box, PdfPagePixelBox::new(600, 1550, 1200, 2325));
    Ok(())
}

#[test]
fn document_extract_pdf_render_region_manifest_preserves_provenance() -> Result<(), String> {
    let page_manifest = sample_manifest(0);
    let region_manifest = sample_region_manifest()?;

    assert_eq!(region_manifest.shard_type, PdfOcrShardType::Region);
    assert_eq!(region_manifest.region_index, 7);
    assert_eq!(
        region_manifest.parent_shard_element_id,
        page_manifest.element_id
    );
    assert_eq!(region_manifest.reading_order_key, "000002.000007");
    assert_eq!(
        region_manifest.source_page_pixel_box,
        PdfPagePixelBox::new(600, 1550, 1200, 2325)
    );
    assert_close(region_manifest.geometry.crop_box.left, 162.0);
    assert_ne!(region_manifest.element_id, page_manifest.element_id);
    Ok(())
}

#[test]
fn document_extract_pdf_render_region_request_defaults_reading_order_key() {
    let request =
        PdfPageRegionRenderRequest::new(12, 34, PdfPageBox::new(10.0, 20.0, 30.0, 40.0), None);

    assert_eq!(request.effective_reading_order_key(), "000012.000034");
}

#[test]
fn document_extract_pdf_render_writes_region_crop_image_identity() -> Result<(), String> {
    let image = DynamicImage::ImageRgba8(ImageBuffer::from_fn(10, 8, |x, y| {
        Rgba([
            u8::try_from(x).unwrap_or_default(),
            u8::try_from(y).unwrap_or_default(),
            128,
            255,
        ])
    }));
    let temp_dir = tempfile::tempdir().map_err(|error| error.to_string())?;
    let crop_path = temp_dir.path().join("region.png");

    let identity = save_region_crop_image(
        &image,
        PdfPagePixelBox::new(2, 1, 7, 5),
        crop_path.as_path(),
    )?;

    assert_eq!(identity.width_px, 5);
    assert_eq!(identity.height_px, 4);
    assert_eq!(identity.sha256.len(), 64);
    assert!(identity.path.is_file());
    let cropped = image::open(identity.path.as_path()).map_err(|error| error.to_string())?;
    assert_eq!(cropped.width(), 5);
    assert_eq!(cropped.height(), 4);
    Ok(())
}

#[test]
fn document_extract_pdf_render_shard_id_is_content_addressed() {
    let first = sample_manifest(0);
    let second = sample_manifest(0);
    let mut different = sample_manifest(0);
    different.page_index = 3;
    different.element_id = shard_element_id(
        different.source_content_hash.as_str(),
        different.page_index,
        different.render_profile.as_str(),
    );

    assert_eq!(first.element_id, second.element_id);
    assert_ne!(first.element_id, different.element_id);
}
