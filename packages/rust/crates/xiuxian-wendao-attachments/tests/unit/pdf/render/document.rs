use super::{PdfRegionRenderMode, pdf_region_render_mode_from_value};

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
