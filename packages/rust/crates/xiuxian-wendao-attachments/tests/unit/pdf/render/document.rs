use pdfium_render::prelude::{PdfPageRenderRotation, PdfiumError};

use super::{
    PdfRegionRenderMode, pdf_region_render_mode_from_value, pdf_rotation_degrees_from_result,
};

#[test]
fn pdf_region_render_mode_accepts_direct_crop_spelling_variants() {
    assert_eq!(
        pdf_region_render_mode_from_value(Some("direct-crop")),
        PdfRegionRenderMode::DirectCrop
    );
    assert_eq!(
        pdf_region_render_mode_from_value(Some(" DIRECT_CROP ")),
        PdfRegionRenderMode::DirectCrop
    );
}

#[test]
fn pdf_region_render_mode_rejects_unknown_values() {
    assert_eq!(
        pdf_region_render_mode_from_value(None),
        PdfRegionRenderMode::Default
    );
    assert_eq!(
        pdf_region_render_mode_from_value(Some("clip-union")),
        PdfRegionRenderMode::Default
    );
    assert_eq!(
        pdf_region_render_mode_from_value(Some("direct-crop-parallel")),
        PdfRegionRenderMode::Default
    );
}

#[test]
fn pdf_rotation_degrees_defaults_unknown_bitmap_rotation_to_zero() {
    assert_eq!(
        pdf_rotation_degrees_from_result(Ok(PdfPageRenderRotation::Degrees90), 7),
        Ok(90)
    );
    assert_eq!(
        pdf_rotation_degrees_from_result(Err(PdfiumError::UnknownBitmapRotation), 7),
        Ok(0)
    );
}
